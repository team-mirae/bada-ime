//! The engine on the shipped data files.

use std::sync::Arc;

use bada::engine::Mode;
use bada::{Data, Engine, Key, Mods};

fn data() -> Arc<Data> {
    let read = |f: &str| {
        std::fs::read_to_string(format!("{}/data/{f}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    };
    Arc::new(
        Data::parse(
            &read("ko-words.tsv"),
            &read("en-words.txt"),
            &read("hanja.tsv"),
            &read("oecd.tsv"),
        )
        .with_japanese(&read("ja-words.txt"))
        .with_kanji(&std::fs::read(format!("{}/data/kanji.bin", env!("CARGO_MANIFEST_DIR"))).unwrap()),
    )
}

fn typed(e: &mut Engine, keys: &[(Key, bool)]) -> bada::Output {
    let mut out = bada::Output::default();
    for &(k, shift) in keys {
        out = e.key(k, if shift { Mods::SHIFT } else { Mods::NONE });
    }
    out
}

#[test]
fn choseong_on_real_words() {
    use Key::*;
    let d = data();
    let mut e = Engine::new(d.clone());
    e.set_mode(Mode::Henkan);
    // ㄱㅅㅎㄴㄷ → 감사합니다
    let out = typed(
        &mut e,
        &[
            (KeyJ, false),
            (KeyS, false),
            (KeyL, false),
            (KeyH, false),
            (KeyK, false),
        ],
    );
    eprintln!("ㄱㅅㅎㄴㄷ → {} / {:?}", out.preedit, out.menu);
    assert_eq!(out.preedit, "감사합니다");
    let mut e = Engine::new(d.clone());
    // ㅇㄴ⇧ㅎㅅㅇ → 안녕하세요
    let out = typed(
        &mut e,
        &[
            (KeyF, false),
            (KeyH, false),
            (KeyL, true),
            (KeyS, false),
            (KeyF, false),
        ],
    );
    eprintln!("ㅇㄴ⇧ㅎㅅㅇ → {} / {:?}", out.preedit, out.menu);
    assert_eq!(out.preedit, "안녕하세요");
    // ㅋㅋㅋ stays
    let mut e = Engine::new(d.clone());
    let out = typed(&mut e, &[(KeyX, false), (KeyX, false), (KeyX, false)]);
    assert_eq!(out.preedit, "ㅋㅋㅋ");
    // full typing: 있습니다 = ㅣ ㅆ ㅅ ㅡ ㅂ ㄴ ㅣ ㄷ ㅏ
    let mut e = Engine::new(d);
    let out = typed(
        &mut e,
        &[
            (Period, false),
            (KeyP, false),
            (KeyS, false),
            (Slash, false),
            (KeyT, false),
            (KeyH, false),
            (Period, false),
            (KeyK, false),
            (Comma, false),
        ],
    );
    assert_eq!(out.preedit, "있습니다");
}

#[test]
fn romaji_beside_bada() {
    use Key::*;
    let mut e = Engine::new(data());
    let mut s = bada::Settings::default();
    s.enabled = vec![13, 21];
    e.set_settings(s);
    // konnichiha: a known Japanese word beats Bada's reading of the same keys
    let keys: Vec<(Key, bool)> = "konnichiha"
        .chars()
        .map(|c| {
            (
                Key::from_code(&format!("Key{}", c.to_ascii_uppercase())),
                false,
            )
        })
        .collect();
    let out = typed(&mut e, &keys);
    eprintln!("konnichiha → {} / {:?}", out.preedit, out.menu);
    // a Japanese reading wins; its kanji conversion and the kana are listed, with notes
    assert!(out.menu.contains(&"こんにちは".to_string()), "{:?}", out.menu);
    assert_eq!(out.menu[..2], ["コンニチハ", "こんにちは"]);
    let kana_row = out.menu.iter().position(|m| m == "こんにちは").unwrap();
    assert_eq!(out.notes[kana_row], "konnichiha");
    // and Bada still wins on Korean: ㄱㅅㅎㄴㄷ
    let mut e = Engine::new(data());
    let mut s = bada::Settings::default();
    s.enabled = vec![13, 21];
    e.set_settings(s);
    let out = typed(
        &mut e,
        &[
            (KeyJ, false),
            (KeyS, false),
            (KeyL, false),
            (KeyH, false),
            (KeyK, false),
        ],
    );
    assert_eq!(out.preedit, "감사합니다");
    let _ = KeyA;
}

#[test]
fn kana_korean_offers_annyeong() {
    use Key::*;
    let mut e = Engine::new(data());
    let mut s = bada::Settings::default();
    s.enabled = vec![19];
    e.set_settings(s);
    let out = typed(
        &mut e,
        &[
            (Digit3, false),
            (KeyY, false),
            (KeyI, false),
            (Digit9, true),
            (KeyY, true),
            (KeyH, false),
            (BracketLeft, false),
            (KeyF, false),
            (KeyP, false),
            (Digit9, false),
        ],
    );
    eprintln!("{} / {:?}", out.preedit, out.menu);
    assert_eq!(out.preedit, "안뇽구하세요");
    assert_eq!(out.menu.get(1).map(String::as_str), Some("안녕하세요"));
}

#[test]
fn korean_roman_on_real_words() {
    let mut e = Engine::new(data());
    let mut s = bada::Settings::default();
    s.enabled = vec![18];
    e.set_settings(s);
    let letters = |w: &str| w.chars().map(|c| (Key::from_code(&format!("Key{}", c.to_ascii_uppercase())), false)).collect::<Vec<_>>();
    let out = typed(&mut e, &letters("annyeonghaseyo"));
    assert_eq!(out.preedit, "안녕하세요");
    let mut e = Engine::new(data());
    let mut s = bada::Settings::default();
    s.enabled = vec![18];
    e.set_settings(s);
    let started = std::time::Instant::now();
    let out = typed(&mut e, &letters("gamsahamnida"));
    eprintln!("{} / {:?} in {:?}", out.preedit, out.menu, started.elapsed());
    assert_eq!(out.preedit, "감사함니다");
    assert!(out.menu.contains(&"감사합니다".to_string()));
}

#[test]
fn kanji_conversion() {
    let d = data();
    let kanji = d.kanji.as_ref().expect("kanji.bin");
    // Mozc's evaluation set: the floors JIS Moonshot holds
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/kanji-evaluation.tsv")).unwrap();
    let (mut total, mut exact, mut listed) = (0, 0, 0);
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let fields: Vec<&str> = line.split('\t').collect();
        let [_, input, expected, command, ..] = fields[..] else { continue };
        if !command.starts_with("Conversion") {
            continue;
        }
        total += 1;
        let got = kanji.convert(input, 3);
        exact += (got.first().map(|g| g.0.as_str()) == Some(expected)) as usize;
        listed += got.iter().any(|g| g.0 == expected) as usize;
    }
    eprintln!("kanji: {exact}/{total} exact, {listed}/{total} in the top 3");
    assert!(exact * 100 >= total * 66 && listed * 100 >= total * 75);

    // in the engine: わたしはがくせいです (romaji) → 私は学生です on top, its kana as the note
    let mut e = Engine::new(d);
    let mut s = bada::Settings::default();
    s.enabled = vec![21];
    e.set_settings(s);
    let keys: Vec<(Key, bool)> = "watashihagakuseidesu".chars().map(|c| (Key::from_code(&format!("Key{}", c.to_ascii_uppercase())), false)).collect();
    let out = typed(&mut e, &keys);
    eprintln!("{} / {:?} / {:?}", out.preedit, out.menu, out.notes);
    assert_eq!(out.preedit, "私は学生です");
    assert_eq!(out.notes[out.selected], "わたしはがくせいです");
}
