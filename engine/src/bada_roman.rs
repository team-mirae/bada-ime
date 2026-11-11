//! Layout 15, bada roman (docs/layouts/15-bada-roman.md): Korean typed as it's romanized, as in
//! Korean Roman (18), with keys of its own: `q` ch, `x` an initial ㅇ, `;` a syllable break, `-`
//! after e ㅐ / ㅒ (e- 애); `woo` ㅜ and a final `h` ㅎ. The other number and symbol keys type their
//! JIS characters on both layers (¥ is ₩; ⇧0 +, ⇧[ 「, ⇧] 」). `yu` and `u` before a final also read ㅕ
//! and ㅓ (nyun 년, sung 성), and a Japanese on'yomi types its Sino-Korean syllable where that's
//! shorter (mei 명: `engine/data/ja-shortcuts.tsv`), mixed with Korean spelling (seigong 성공).
//!
//! The default reading comes first (the longest onset for each next syllable, as in 18); every
//! other reading follows, for the engine to rank by word counts.

use std::sync::OnceLock;

use crate::hangul::{compose, decompose};
use crate::key::Key;
use crate::layout::{Press, jis_char};

// Bytes for the keys that aren't letters (the letters are lowercase).
const IEUNG: u8 = b'X';
const LONG: u8 = b'L';

/// Onsets: (spelling, jamo, the default first).
const ONSETS: &[(&str, &[char])] = &[
    ("kk", &['ㄲ']), ("gg", &['ㄲ']), ("tt", &['ㄸ']), ("dd", &['ㄸ']), ("pp", &['ㅃ']), ("bb", &['ㅃ']),
    ("ss", &['ㅆ']), ("jj", &['ㅉ']), ("ch", &['ㅊ', 'ㅈ']), ("sh", &['ㅅ']),
    ("k", &['ㅋ', 'ㄱ']), ("g", &['ㄱ']), ("n", &['ㄴ']), ("t", &['ㅌ', 'ㄷ']), ("d", &['ㄷ']),
    ("r", &['ㄹ']), ("l", &['ㄹ']), ("m", &['ㅁ']), ("p", &['ㅍ', 'ㅂ']), ("b", &['ㅂ']),
    ("s", &['ㅅ', 'ㅆ']), ("j", &['ㅈ']), ("c", &['ㅊ']), ("h", &['ㅎ']), ("f", &['ㅍ']), ("v", &['ㅂ']), ("z", &['ㅈ']),
    ("X", &['ㅇ']),
];

const VOWELS: &[(&str, &[char])] = &[
    ("yae", &['ㅒ', 'ㅖ']), ("wae", &['ㅙ', 'ㅞ', 'ㅚ']), ("yeo", &['ㅕ']), ("weo", &['ㅝ']), ("woo", &['ㅜ']), ("you", &['ㅠ']), ("eui", &['ㅢ']), ("yeL", &['ㅒ']),
    ("ae", &['ㅐ', 'ㅔ']), ("ya", &['ㅑ']), ("eo", &['ㅓ']), ("ye", &['ㅖ', 'ㅒ']), ("wa", &['ㅘ']),
    ("oe", &['ㅚ', 'ㅙ', 'ㅞ']), ("yo", &['ㅛ', 'ㅕ']), ("wo", &['ㅝ']), ("we", &['ㅞ', 'ㅙ', 'ㅚ']),
    ("wi", &['ㅟ']), ("yu", &['ㅠ']), ("eu", &['ㅡ']), ("ui", &['ㅢ']), ("oo", &['ㅜ']), ("ee", &['ㅣ']),
    ("eL", &['ㅐ']),
    ("a", &['ㅏ']), ("e", &['ㅔ', 'ㅐ']), ("o", &['ㅗ', 'ㅓ']), ("u", &['ㅜ', 'ㅡ']), ("i", &['ㅣ']),
];

/// Interlaced vowels: before a final, `yu` and `you` also read ㅕ (the usual one: nyun 년, young 영) and
/// `u` ㅓ (sung 성).
const YU_BEFORE_FINAL: &[char] = &['ㅕ', 'ㅠ'];
const U_BEFORE_FINAL: &[char] = &['ㅜ', 'ㅡ', 'ㅓ'];

const CODAS: &[(&str, &[char])] = &[
    ("ng", &['ㅇ', 'ㄱ']), ("kk", &['ㄲ']), ("ss", &['ㅆ']), ("ts", &['ㅆ']),
    ("lk", &['ㄺ']), ("lg", &['ㄺ']), ("lm", &['ㄻ']), ("lb", &['ㄼ']), ("lp", &['ㄼ']), ("lh", &['ㅀ']),
    ("ps", &['ㅄ']), ("bs", &['ㅄ']), ("nh", &['ㄶ']), ("nj", &['ㄵ']), ("ks", &['ㄳ']),
    ("k", &['ㄱ', 'ㅋ', 'ㄲ']), ("g", &['ㄱ']), ("n", &['ㄴ']), ("t", &['ㅅ', 'ㄷ', 'ㅆ', 'ㅈ', 'ㅊ', 'ㅌ', 'ㅎ']),
    ("d", &['ㄷ']), ("l", &['ㄹ']), ("r", &['ㄹ']), ("m", &['ㅁ', 'ㅂ']), ("p", &['ㅂ', 'ㅍ']), ("b", &['ㅂ']),
    ("s", &['ㅅ']), ("h", &['ㅎ']),
];

/// The romaji shortcuts (`ja-shortcuts.tsv`): romaji → its syllables, the most frequent first.
fn shortcuts() -> &'static [(String, Vec<char>)] {
    static TABLE: OnceLock<Vec<(String, Vec<char>)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut out: Vec<(String, Vec<char>)> = Vec::new();
        for line in include_str!("../data/ja-shortcuts.tsv").lines().filter(|l| !l.starts_with('#')) {
            let mut f = line.split('\t');
            let (Some(romaji), Some(syl)) = (f.next(), f.next().and_then(|s| s.chars().next())) else { continue };
            match out.iter_mut().find(|(r, _)| r == romaji) {
                Some((_, s)) => s.push(syl),
                None => out.push((romaji.to_string(), vec![syl])),
            }
        }
        out
    })
}

fn is_vowel_byte(c: u8) -> bool {
    b"aeiouwyL".contains(&c)
}

fn matches<'a>(table: &'a [(&'a str, &'a [char])], s: &[u8]) -> Vec<(&'a str, &'a [char])> {
    table.iter().filter(|(sp, _)| s.starts_with(sp.as_bytes())).map(|(sp, j)| (*sp, *j)).collect()
}

#[derive(Debug, PartialEq, Eq)]
enum Piece {
    Run(Vec<u8>),
    Break,
    Lit(char),
}

/// What a press means here: a byte of a run, a break, or a character as it is.
fn meaning(p: Press, run: Option<&[u8]>) -> Option<Piece> {
    let c = jis_char(p.key, p.shift)?;
    if p.key == Key::Space {
        return None;
    }
    if p.key.is_letter() {
        if p.alt {
            return None;
        }
        return Some(Piece::Run(match c.to_ascii_lowercase() {
            'x' => vec![IEUNG],
            'q' => b"ch".to_vec(), // q is ch (ㅊ)
            c => vec![c as u8],
        }));
    }
    if p.alt {
        // ⌥; is the middle dot ·. ⇧⌥: the key's shifted JIS character. Other ⌥ keys aren't taken.
        if !p.shift && p.key == Key::Semicolon {
            return Some(Piece::Lit('·'));
        }
        return p.shift.then_some(Piece::Lit(c));
    }
    if p.shift {
        // ⇧: the key's shifted JIS character, but 0 +, [ 「, ] 」 and ; itself.
        return Some(Piece::Lit(match p.key {
            Key::Digit0 => '+',
            Key::BracketRight => '「',
            Key::Backslash => '」',
            Key::Semicolon => ';',
            _ => c,
        }));
    }
    Some(match p.key {
        Key::Semicolon => Piece::Break,
        Key::Minus if run.and_then(|r| r.last()) == Some(&b'e') => Piece::Run(vec![LONG]),
        // The base layer: the key's JIS character (1–0, - ^ @ [ : ] , . / \\), but ¥ is ₩.
        Key::IntlYen => Piece::Lit('₩'),
        _ => Piece::Lit(c),
    })
}

fn pieces(presses: &[Press]) -> Vec<Piece> {
    let mut out: Vec<Piece> = Vec::new();
    for &p in presses {
        let run = match out.last() {
            Some(Piece::Run(r)) => Some(r.as_slice()),
            _ => None,
        };
        match meaning(p, run) {
            Some(Piece::Run(b)) => match out.last_mut() {
                Some(Piece::Run(r)) => r.extend(b),
                _ => out.push(Piece::Run(b)),
            },
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

/// The text a key types here, as the layout file draws it (`docs/layouts/15-bada-roman.json`).
pub fn token(key: Key, shift: bool) -> Option<(String, &'static str)> {
    let p = Press::new(key, shift);
    Some(match meaning(p, Some(b"e"))? {
        Piece::Run(b) if b.len() > 1 => {
            let role = if is_vowel_byte(b[0]) { "v" } else { "l" };
            (String::from_utf8(b).unwrap_or_default(), role)
        }
        Piece::Run(b) => match b[0] {
            IEUNG => ("ㅇ".into(), "i"),
            LONG => ("-".into(), "x"),
            c => ((if shift { c.to_ascii_uppercase() } else { c } as char).to_string(), "l"),
        },
        Piece::Break => (";".into(), "x"),
        Piece::Lit(c) => (c.to_string(), "x"),
    })
}

/// The character a press types as it is (punctuation, a ⇧ digit), after the staged presses; None
/// for a press that's part of a word. The engine commits what shows before such a character.
pub fn literal(staged: &[Press], p: Press) -> Option<char> {
    let parts = pieces(staged);
    let run = match parts.last() {
        Some(Piece::Run(r)) => Some(r.as_slice()),
        _ => None,
    };
    match meaning(p, run)? {
        Piece::Lit(c) => Some(c),
        _ => None,
    }
}

pub fn takes(p: Press) -> bool {
    meaning(p, Some(b"e")).is_some()
}

/// One way to read a syllable at a position: its syllables (each a reading) and where it ends.
struct Opt {
    syls: Vec<char>,
    end: usize,
}

/// Every way a syllable can start at `i` in `run`, the default first; then the romaji shortcuts.
fn syllables_at(run: &[u8], i: usize, with_shortcuts: bool) -> Vec<Opt> {
    let mut out = Vec::new();
    let mut onsets: Vec<(&str, &[char])> = matches(ONSETS, &run[i..]);
    if run.get(i).is_some_and(|&c| is_vowel_byte(c) && c != LONG) {
        onsets.push(("", &['ㅇ'][..]));
    }
    for (os, onset) in onsets {
        let j = i + os.len();
        for (vs, vowel) in matches(VOWELS, &run[j..]) {
            let k = j + vs.len();
            let run_end = (k..run.len()).find(|&x| is_vowel_byte(run[x])).unwrap_or(run.len());
            let cons = &run[k..run_end];
            let mut codas: Vec<(&str, &[char])> = matches(CODAS, cons);
            codas.push(("", &[][..]));
            let at_end = run_end == run.len();
            let preferred = |len: usize| -> bool {
                let rest = &cons[len..];
                rest.is_empty() || (!at_end && ONSETS.iter().any(|(sp, _)| sp.as_bytes() == rest))
            };
            codas.sort_by_key(|&(sp, _)| {
                let rank = if preferred(sp.len()) { 0 } else { 1 };
                (rank, if rank == 0 { sp.len() } else { usize::MAX - sp.len() })
            });
            for (cs, coda) in codas {
                let vowel = match (vs, cs.is_empty()) {
                    ("yu" | "you", false) => YU_BEFORE_FINAL,
                    ("u", false) => U_BEFORE_FINAL,
                    _ => vowel,
                };
                let finals: Vec<Option<char>> = if coda.is_empty() { vec![None] } else { coda.iter().map(|&c| Some(c)).collect() };
                let mut syls = Vec::new();
                for &l in onset {
                    for &v in vowel {
                        for &f in &finals {
                            if let Some(s) = compose(l, v, f) {
                                syls.push(s);
                            }
                        }
                    }
                }
                if !syls.is_empty() {
                    out.push(Opt { syls, end: k + cs.len() });
                }
            }
        }
    }
    if with_shortcuts {
        for (romaji, syls) in shortcuts() {
            if run[i..].starts_with(romaji.as_bytes()) {
                out.push(Opt { syls: syls.clone(), end: i + romaji.len() });
            }
        }
    }
    out
}

/// Readings of the presses, the default first, at most `limit`. `shortcuts`: the romaji shortcuts
/// (the setting "Accept Japanese romaji readings as shortcuts").
pub fn readings(presses: &[Press], limit: usize, shortcuts: bool) -> Vec<String> {
    let parts = pieces(presses);
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut budget = 200_000usize;
    walk(&parts, 0, 0, shortcuts, &mut String::new(), &mut out, &mut seen, limit, &mut budget);
    if out.is_empty() {
        out.push(loose(&parts));
    }
    out
}

pub fn compose_presses(presses: &[Press]) -> String {
    readings(presses, 1, false).into_iter().next().unwrap_or_default()
}

/// The presses as an English word, if they're letters only: (lowercase word, capitalized).
/// Capitalized: only the first letter typed with Shift. All caps: None (not settled yet).
pub fn latin_word(presses: &[Press]) -> Option<(String, bool)> {
    if presses.is_empty() || !presses.iter().all(|p| p.key.is_letter() && !p.alt) {
        return None;
    }
    let word: String = presses.iter().filter_map(|p| jis_char(p.key, false)).collect();
    let shifted = presses.iter().filter(|p| p.shift).count();
    match shifted {
        0 => Some((word, false)),
        1 if presses[0].shift => Some((word, true)),
        _ => None,
    }
}

/// A byte that doesn't make a syllable, as it shows.
fn loose_char(c: u8) -> char {
    match c {
        IEUNG => 'ㅇ',
        LONG => '-',
        c => matches(ONSETS, &[c]).first().map_or(c as char, |(_, j)| j[0]),
    }
}

fn loose(parts: &[Piece]) -> String {
    let mut s = String::new();
    for p in parts {
        match p {
            Piece::Run(run) => {
                let mut i = 0;
                while i < run.len() {
                    if let Some(o) = syllables_at(run, i, false).into_iter().next() {
                        s.push(o.syls[0]);
                        i = o.end;
                        continue;
                    }
                    s.push(loose_char(run[i]));
                    i += 1;
                }
            }
            Piece::Break => {}
            Piece::Lit(c) => s.push(*c),
        }
    }
    s
}

#[allow(clippy::too_many_arguments)]
fn walk(parts: &[Piece], pi: usize, i: usize, shortcuts: bool, built: &mut String, out: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>, limit: usize, budget: &mut usize) {
    if out.len() >= limit || *budget == 0 {
        return;
    }
    *budget -= 1;
    let Some(part) = parts.get(pi) else {
        if seen.insert(built.clone()) {
            out.push(built.clone());
        }
        return;
    };
    match part {
        Piece::Break => walk(parts, pi + 1, 0, shortcuts, built, out, seen, limit, budget),
        Piece::Lit(c) => {
            built.push(*c);
            walk(parts, pi + 1, 0, shortcuts, built, out, seen, limit, budget);
            built.pop();
        }
        Piece::Run(run) => {
            if i == run.len() {
                return walk(parts, pi + 1, 0, shortcuts, built, out, seen, limit, budget);
            }
            for o in syllables_at(run, i, shortcuts) {
                for &s in &o.syls {
                    built.push(s);
                    walk(parts, pi, o.end, shortcuts, built, out, seen, limit, budget);
                    built.pop();
                }
            }
        }
    }
}

/// Which jamo of each syllable an F-key action keeps (docs/layouts/15-bada-roman.md, F-key actions).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    Initials,
    Vowels,
    Finals,
}

/// Leaves one jamo of each Hangul syllable (a syllable with no final is dropped for `Finals`);
/// everything else stays as it is.
pub fn keep_jamo(text: &str, keep: Keep) -> String {
    text.chars()
        .filter_map(|c| match decompose(c) {
            None => Some(c),
            Some((l, v, f)) => match keep {
                Keep::Initials => Some(l),
                Keep::Vowels => Some(v),
                Keep::Finals => f,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Presses for text: letters (uppercase: with Shift), and `,` `.` `:` `@` `;` `-` as their keys.
    fn presses(text: &str) -> Vec<Press> {
        text.chars()
            .map(|c| match c {
                ',' => Press::new(Key::Comma, false),
                '.' => Press::new(Key::Period, false),
                ':' => Press::new(Key::Quote, false),
                '@' => Press::new(Key::BracketLeft, false),
                ';' => Press::new(Key::Semicolon, false),
                '-' => Press::new(Key::Minus, false),
                '1' => Press::new(Key::Digit1, false),
                '0' => Press::new(Key::Digit0, false),
                '9' => Press::new(Key::Digit9, false),
                '8' => Press::new(Key::Digit8, false),
                '!' => Press::new(Key::Digit1, true),
                c => Press::new(Key::from_code(&format!("Key{}", c.to_ascii_uppercase())), c.is_ascii_uppercase()),
            })
            .collect()
    }
    fn default(text: &str) -> String {
        compose_presses(&presses(text))
    }
    fn all(text: &str) -> Vec<String> {
        readings(&presses(text), 100_000, true)
    }

    #[test]
    fn keys() {
        assert_eq!(default("hanguk"), "한국");
        assert_eq!(default("han;guk"), "한국");
        assert_eq!(default("hang;uk"), "항욱");
        assert_eq!(default("anxa"), "안아");
        assert_eq!(default("xx"), "ㅇㅇ");
        assert_eq!(default("e-"), "애");
        assert_eq!(default("pe-"), "패");
        assert_eq!(default("ye-"), "얘");
        assert_eq!(default("Hanguk"), "한국");
        // the number and symbol keys: JIS on both layers, but ¥ ₩, ⇧0 +, ⇧[ 「, ⇧] 」, ⇧; ;
        let base = |k| compose_presses(&[Press::new(k, false)]);
        let shifted = |k| compose_presses(&[Press::new(k, true)]);
        let row = [Key::Digit1, Key::Digit2, Key::Digit3, Key::Digit4, Key::Digit5, Key::Digit6, Key::Digit7, Key::Digit8,
                   Key::Digit9, Key::Digit0, Key::Minus, Key::Equal, Key::IntlYen];
        assert_eq!(row.map(base), ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "^", "₩"]);
        assert_eq!(row.map(shifted), ["!", "\"", "#", "$", "%", "&", "'", "(", ")", "+", "=", "~", "|"]);
        let keys = [Key::BracketLeft, Key::BracketRight, Key::Quote, Key::Backslash, Key::Comma, Key::Period, Key::Slash, Key::IntlRo, Key::Semicolon];
        assert_eq!(keys.map(base), ["@", "[", ":", "]", ",", ".", "/", "\\", ""]);
        assert_eq!(keys.map(shifted), ["`", "「", "*", "」", "<", ">", "?", "_", ";"]);
        assert_eq!(default("qa"), "차");
        assert_eq!(default("qeong"), "청");
        assert_eq!(default("Qa"), "차");
        // woo ㅜ, a final h
        assert_eq!(default("woori"), "우리");
        assert_eq!(default("young"), "영");
        // eui: 의, as names and older romanizations write it
        assert_eq!(default("eui"), "의");
        assert_eq!(default("gaeui"), "개의");
        assert_eq!(default("you"), "유");
        assert_eq!(default("youngho"), "영호");
        assert_eq!(default("joh"), "좋");
        // t (a final ㅅ) and s again: ㅆ
        assert_eq!(default("its"), "있");
        assert_eq!(default("hats"), "핬");
        assert!(all("itseo").contains(&"있어".to_string()));
        assert_eq!(default("zoh"), "좋");
        assert!(all("joha").contains(&"좋아".to_string()));
        assert_eq!(default("joha"), "조하");
        // ⇧⌥: the shifted character; ⌥ alone isn't taken
        assert_eq!(compose_presses(&[Press { key: Key::Digit1, shift: true, alt: true }]), "!");
        assert!(!takes(Press { key: Key::Digit1, shift: false, alt: true }));
    }

    #[test]
    fn interlaced_vowels() {
        assert_eq!(default("nyun"), "년");
        assert_eq!(default("gyung"), "경");
        assert!(all("kyung").contains(&"경".to_string()));
        assert!(all("nyun").contains(&"뉸".to_string()));
        assert_eq!(default("nyu"), "뉴");
        assert!(all("sung").contains(&"성".to_string()));
        assert_eq!(default("sung"), "숭");
    }

    #[test]
    fn shortcuts_mix() {
        assert!(all("mei").contains(&"명".to_string()));
        assert!(all("jou").contains(&"정".to_string()));
        assert!(all("seigong").contains(&"성공".to_string()));
        assert!(all("meipe-").contains(&"명패".to_string()));
        assert!(all("kenkang").contains(&"건강".to_string()));
        // off: not offered
        assert!(!readings(&presses("mei"), 100_000, false).contains(&"명".to_string()));
        // the default reading stays the letters'
        assert_eq!(default("seigong"), "세이공");
    }

    #[test]
    fn english_words() {
        assert_eq!(latin_word(&presses("apple")), Some(("apple".into(), false)));
        assert_eq!(latin_word(&presses("Apple")), Some(("apple".into(), true)));
        assert_eq!(latin_word(&presses("APPLE")), None);
        assert_eq!(latin_word(&presses("ap,")), None);
    }

    #[test]
    fn keep() {
        assert_eq!(keep_jamo("한국", Keep::Initials), "ㅎㄱ");
        assert_eq!(keep_jamo("안녕!", Keep::Initials), "ㅇㄴ!");
        assert_eq!(keep_jamo("과자", Keep::Vowels), "ㅘㅏ");
        assert_eq!(keep_jamo("한국", Keep::Finals), "ㄴㄱ");
        assert_eq!(keep_jamo("닭고기", Keep::Finals), "ㄺ");
        assert_eq!(keep_jamo("가나 abc", Keep::Finals), " abc");
        assert_eq!(keep_jamo("ㅋㅋ", Keep::Initials), "ㅋㅋ");
    }
}
