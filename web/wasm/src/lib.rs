//! bada ime's engine for the web simulator (`web/`). One engine per page, behind a raw
//! WebAssembly ABI: the page copies bytes in with `alloc`, calls `call(op, ptr, len)` and reads the
//! reply (UTF-8 JSON or text) at `reply_ptr()` / `reply_len()`. The ops mirror the C ABI in
//! `engine/include/bada.h` that the macOS shell uses, so the page behaves as the Mac app does.

use std::cell::RefCell;
use std::sync::Arc;

use bada::engine::{ACTIONS, Mode};
use bada::layout::CATALOG;
use bada::{Data, Engine, Key, Mods, Settings};
use serde::Deserialize;

/// The data files, in `data_put`'s slot order.
const SLOTS: usize = 10;
const KO: usize = 0;
const EN: usize = 1;
const HANJA: usize = 2;
const OECD: usize = 3;
const JA: usize = 4;
const KANJI: usize = 5;
const LM: usize = 6;
const SYMBOLS: usize = 7;
const EMOJI: usize = 8;
const UNICODE: usize = 9;

thread_local! {
    static FILES: RefCell<Vec<Vec<u8>>> = RefCell::new(vec![Vec::new(); SLOTS]);
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
    static REPLY: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// A buffer of `len` bytes for the page to write into; pass it on to `data_put` or `call`.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len.max(1));
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Frees a buffer from `alloc` that wasn't handed to `data_put` or `call`.
#[unsafe(no_mangle)]
pub extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    // SAFETY: from `alloc(len)`.
    drop(unsafe { Vec::from_raw_parts(ptr, 0, len.max(1)) });
}

/// Takes `len` bytes at `ptr` (from `alloc(len)`) as data file `slot`.
#[unsafe(no_mangle)]
pub extern "C" fn data_put(slot: usize, ptr: *mut u8, len: usize) {
    // SAFETY: from `alloc(len)`, filled by the page.
    let bytes = unsafe { Vec::from_raw_parts(ptr, len, len.max(1)) };
    if slot < SLOTS {
        FILES.with(|f| f.borrow_mut()[slot] = bytes);
    }
}

/// Parses the data files given so far (missing ones are empty) and starts the engine.
#[unsafe(no_mangle)]
pub extern "C" fn init() {
    let files = FILES.with(|f| std::mem::replace(&mut *f.borrow_mut(), vec![Vec::new(); SLOTS]));
    let text = |slot: usize| std::str::from_utf8(&files[slot]).unwrap_or("");
    let data = Data::parse(text(KO), text(EN), text(HANJA), text(OECD))
        .with_japanese(text(JA))
        .with_kanji(&files[KANJI])
        .with_lm(&files[LM])
        .with_search(text(SYMBOLS), text(EMOJI), text(UNICODE));
    let engine = Engine::new(Arc::new(data));
    ENGINE.with(|e| *e.borrow_mut() = Some(engine));
}

#[unsafe(no_mangle)]
pub extern "C" fn reply_ptr() -> *const u8 {
    REPLY.with(|r| r.borrow().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn reply_len() -> usize {
    REPLY.with(|r| r.borrow().len())
}

#[derive(Deserialize)]
struct KeyEvent {
    key: String,
    #[serde(flatten)]
    mods: Mods,
}

fn json(value: impl serde::Serialize) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| "{}".into())
}

fn with<T: serde::Serialize + Default>(f: impl FnOnce(&mut Engine) -> T) -> String {
    json(ENGINE.with(|e| e.borrow_mut().as_mut().map(f)).unwrap_or_default())
}

/// One operation; `ptr`/`len` (from `alloc(len)`, freed here) is its argument as UTF-8.
///
/// 0 key (event JSON) · 1 set settings (JSON; replies with the settings in effect) · 2 set mode
/// ("0" muhenkan, "1" henkan) · 3 set henkan profile · 4 flush · 5 the host's selection ·
/// 6 catalog · 7 load the user dictionary.
#[unsafe(no_mangle)]
pub extern "C" fn call(op: u32, ptr: *mut u8, len: usize) {
    let arg = if ptr.is_null() {
        String::new()
    } else {
        // SAFETY: from `alloc(len)`, filled by the page.
        let bytes = unsafe { Vec::from_raw_parts(ptr, len, len.max(1)) };
        String::from_utf8(bytes).unwrap_or_default()
    };
    let out = match op {
        0 => match serde_json::from_str::<KeyEvent>(&arg) {
            Ok(event) => with(|e| e.key(Key::from_code(&event.key), event.mods)),
            Err(_) => "{}".into(),
        },
        1 => {
            let settings = serde_json::from_str::<Settings>(&arg).unwrap_or_default();
            with(|e| {
                e.set_settings(settings);
                Some(e.settings().clone())
            })
        }
        2 => {
            let mode = if arg == "0" { Mode::Muhenkan } else { Mode::Henkan };
            with(|e| e.set_mode(mode))
        }
        3 => with(|e| e.set_profile(&arg)),
        4 => with(Engine::flush),
        5 => with(|e| e.host_selection(&arg)),
        6 => {
            let layouts: Vec<_> = CATALOG
                .iter()
                .map(|(id, name, henkan)| serde_json::json!({"id": id, "name": name, "henkan": henkan}))
                .collect();
            let actions: Vec<_> = ACTIONS
                .iter()
                .map(|(m, a, label)| serde_json::json!({"method": m, "action": a, "label": label}))
                .collect();
            json(serde_json::json!({"layouts": layouts, "actions": actions}))
        }
        7 => json(bada::userdict::load(&arg)),
        _ => "null".into(),
    };
    REPLY.with(|r| *r.borrow_mut() = out.into_bytes());
}
