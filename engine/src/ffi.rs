//! The C ABI for the macOS shell (`include/bada.h`). JSON in, JSON out: every call that returns
//! text returns a pointer owned by the engine, valid until the next call on the same engine.

use std::ffi::{CStr, CString, c_char};
use std::sync::Arc;

use serde::Deserialize;

use crate::data::Data;
use crate::engine::{ACTIONS, Engine, Mode, Output, Settings};
use crate::key::{Key, Mods};
use crate::layout::CATALOG;

pub struct Handle {
    engine: Engine,
    last: CString,
}

fn bytes<'a>(ptr: *const u8, len: usize) -> &'a str {
    if ptr.is_null() || len == 0 {
        return "";
    }
    // SAFETY: the shell passes a buffer of `len` bytes that outlives the call.
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(slice).unwrap_or("")
}

fn cstr<'a>(ptr: *const c_char) -> &'a str {
    if ptr.is_null() {
        return "";
    }
    // SAFETY: a NUL-terminated string from the shell, valid for the call.
    unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("")
}

/// Loads the shared data. Free with `bada_data_free` after every engine using it is freed.
#[unsafe(no_mangle)]
pub extern "C" fn bada_data_new(
    ko: *const u8,
    ko_len: usize,
    en: *const u8,
    en_len: usize,
    hanja: *const u8,
    hanja_len: usize,
    oecd: *const u8,
    oecd_len: usize,
    ja: *const u8,
    ja_len: usize,
) -> *const Data {
    let data = Data::parse(
        bytes(ko, ko_len),
        bytes(en, en_len),
        bytes(hanja, hanja_len),
        bytes(oecd, oecd_len),
    )
    .with_japanese(bytes(ja, ja_len));
    Arc::into_raw(Arc::new(data))
}

#[unsafe(no_mangle)]
pub extern "C" fn bada_data_free(data: *const Data) {
    if !data.is_null() {
        // SAFETY: from bada_data_new.
        drop(unsafe { Arc::from_raw(data) });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bada_new(data: *const Data) -> *mut Handle {
    if data.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: from bada_data_new; the engine keeps its own reference.
    let data = unsafe {
        Arc::increment_strong_count(data);
        Arc::from_raw(data)
    };
    Box::into_raw(Box::new(Handle {
        engine: Engine::new(data),
        last: CString::default(),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn bada_free(handle: *mut Handle) {
    if !handle.is_null() {
        // SAFETY: from bada_new.
        drop(unsafe { Box::from_raw(handle) });
    }
}

fn reply(handle: *mut Handle, out: impl serde::Serialize) -> *const c_char {
    // SAFETY: a live handle from bada_new.
    let Some(h) = (unsafe { handle.as_mut() }) else {
        return std::ptr::null();
    };
    let json = serde_json::to_string(&out).unwrap_or_else(|_| "{}".into());
    h.last = CString::new(json).unwrap_or_default();
    h.last.as_ptr()
}

fn with<T>(handle: *mut Handle, f: impl FnOnce(&mut Engine) -> T) -> Option<T> {
    // SAFETY: a live handle from bada_new.
    unsafe { handle.as_mut() }.map(|h| f(&mut h.engine))
}

#[derive(Deserialize)]
struct KeyEvent {
    key: String,
    #[serde(flatten)]
    mods: Mods,
}

/// `{"key":"KeyA","shift":false,"ctrl":false,"alt":false,"meta":false}` → Output JSON.
#[unsafe(no_mangle)]
pub extern "C" fn bada_key(handle: *mut Handle, event: *const c_char) -> *const c_char {
    let out = serde_json::from_str::<KeyEvent>(cstr(event))
        .ok()
        .and_then(|e| with(handle, |engine| engine.key(Key::from_code(&e.key), e.mods)))
        .unwrap_or_default();
    reply(handle, out)
}

/// Settings JSON (engine::Settings). Returns the settings in effect.
#[unsafe(no_mangle)]
pub extern "C" fn bada_set_settings(handle: *mut Handle, json: *const c_char) -> *const c_char {
    let settings = serde_json::from_str::<Settings>(cstr(json)).unwrap_or_default();
    let applied = with(handle, |e| {
        e.set_settings(settings);
        e.settings().clone()
    });
    reply(handle, applied)
}

/// 0 muhenkan, 1 henkan. Returns an Output (what was staged is committed).
#[unsafe(no_mangle)]
pub extern "C" fn bada_set_mode(handle: *mut Handle, mode: u32) -> *const c_char {
    let mode = if mode == 0 {
        Mode::Muhenkan
    } else {
        Mode::Henkan
    };
    let out = with(handle, |e| e.set_mode(mode)).unwrap_or_default();
    reply(handle, out)
}

#[unsafe(no_mangle)]
pub extern "C" fn bada_flush(handle: *mut Handle) -> *const c_char {
    let out = with(handle, Engine::flush).unwrap_or_default();
    reply(handle, out)
}

/// The host's selected text, after an Output with `need_selection`.
#[unsafe(no_mangle)]
pub extern "C" fn bada_host_selection(handle: *mut Handle, text: *const c_char) -> *const c_char {
    let text = cstr(text).to_string();
    let out: Output = with(handle, |e| e.host_selection(&text)).unwrap_or_default();
    reply(handle, out)
}

/// The layout catalog and every method's F-key actions, for the settings window.
#[unsafe(no_mangle)]
pub extern "C" fn bada_catalog(handle: *mut Handle) -> *const c_char {
    let layouts: Vec<_> = CATALOG
        .iter()
        .map(|(id, name, henkan)| serde_json::json!({"id": id, "name": name, "henkan": henkan}))
        .collect();
    let actions: Vec<_> = ACTIONS
        .iter()
        .map(|(m, a, label)| serde_json::json!({"method": m, "action": a, "label": label}))
        .collect();
    reply(
        handle,
        serde_json::json!({"layouts": layouts, "actions": actions}),
    )
}

/// A string the caller frees with `bada_string_free`.
fn owned(text: String) -> *mut c_char {
    CString::new(text).unwrap_or_default().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn bada_string_free(text: *mut c_char) {
    if !text.is_null() {
        // SAFETY: from `owned`.
        drop(unsafe { CString::from_raw(text) });
    }
}

/// 초성 diagnostics (spec §8): on or off for the whole process.
#[unsafe(no_mangle)]
pub extern "C" fn bada_diag_set_enabled(on: u32) {
    crate::diag::set_enabled(on != 0);
}

/// Changes since the last export.
#[unsafe(no_mangle)]
pub extern "C" fn bada_diag_changes() -> u64 {
    crate::diag::changes()
}

/// Everything collected, as JSON; free with `bada_string_free`.
#[unsafe(no_mangle)]
pub extern "C" fn bada_diag_export() -> *mut c_char {
    owned(crate::diag::export())
}

/// Adds saved counts.
#[unsafe(no_mangle)]
pub extern "C" fn bada_diag_import(json: *const c_char) {
    crate::diag::import(cstr(json));
}

#[unsafe(no_mangle)]
pub extern "C" fn bada_diag_clear() {
    crate::diag::clear();
}

/// Loads the user dictionary (docs/dictionary-contract.md) for the whole process; an empty string
/// clears it. Returns a summary, `{"ok":…,"error":…,"positive":…,"negative":…,"shortcuts":…}`;
/// free with `bada_string_free`.
#[unsafe(no_mangle)]
pub extern "C" fn bada_dictionary_load(json: *const c_char) -> *mut c_char {
    owned(serde_json::to_string(&crate::userdict::load(cstr(json))).unwrap_or_default())
}
