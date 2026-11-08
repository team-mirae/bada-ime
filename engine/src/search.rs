//! Layouts 150 Symbols and 151 Unicode (docs/layouts/150-symbols.md, 151-unicode.md): what was typed
//! is a search; the menu lists what it finds under the typed text itself.
//!
//! - **Symbols:** a typed symbol offers its richer forms (`.` → 。 … ・); a letter or digit offers its
//!   variants; words search symbols (by canonical name and tags) and emoji (by name and CLDR
//!   keywords). Every query word must begin a word of the name or tags (`happy face`).
//! - **Unicode:** words search character names; a hex code (`1f600`, `u+1f600`) gives that character.

use std::collections::HashMap;

/// One symbol or emoji.
#[derive(Clone, Debug)]
pub struct Item {
    pub text: String,
    /// Canonical name (shown as the note).
    pub name: String,
    /// Name and tag words, lowercase, for matching.
    words: Vec<String>,
    /// Skin-tone forms: light, medium-light, medium, medium-dark, dark (emoji only).
    tones: Vec<String>,
}

/// Skin tones, in emoji-test.txt's order; "none" is the plain emoji.
pub const TONES: [&str; 6] = ["none", "light", "medium-light", "medium", "medium-dark", "dark"];

#[derive(Default)]
pub struct Search {
    symbols: Vec<Item>,
    /// A typed symbol → its forms (indexes into `symbols`), in file order.
    by_key: HashMap<char, Vec<usize>>,
    emoji: Vec<Item>,
    /// (code point, name lowercase) for every named character.
    unicode: Vec<(u32, String)>,
}

fn words(s: &str) -> Vec<String> {
    s.split(|c: char| c.is_whitespace() || c == '-' || c == '_' || c == '|' || c == ',' || c == ':')
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether every query word begins one of the words.
fn matches(query: &[&str], words: &[String]) -> bool {
    !query.is_empty() && query.iter().all(|q| words.iter().any(|w| w.starts_with(q)))
}

impl Search {
    /// `symbols.tsv`, `emoji.tsv`, `unicode-names.tsv` (any may be empty).
    pub fn parse(symbols: &str, emoji: &str, unicode: &str) -> Search {
        let mut s = Search::default();
        for line in symbols.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            let f: Vec<&str> = line.split('\t').collect();
            let (Some(&text), Some(&name)) = (f.first(), f.get(1)) else { continue };
            let tags = f.get(2).copied().unwrap_or("");
            let at = s.symbols.len();
            for k in f.get(3).copied().unwrap_or("").split(' ').filter_map(|k| k.chars().next()) {
                s.by_key.entry(k).or_default().push(at);
            }
            s.symbols.push(Item { text: text.into(), name: name.into(), words: words(&format!("{name} {tags}")), tones: vec![] });
        }
        for line in emoji.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            let f: Vec<&str> = line.split('\t').collect();
            let (Some(&text), Some(&name)) = (f.first(), f.get(1)) else { continue };
            let keys = f.get(2).copied().unwrap_or("");
            let tones: Vec<String> = f.get(3).copied().unwrap_or("").split(' ').filter(|t| !t.is_empty()).map(String::from).collect();
            s.emoji.push(Item { text: text.into(), name: name.into(), words: words(&format!("{name} {keys}")), tones });
        }
        for line in unicode.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            if let Some((hex, name)) = line.split_once('\t') {
                if let Ok(cp) = u32::from_str_radix(hex, 16) {
                    s.unicode.push((cp, name.to_lowercase()));
                }
            }
        }
        s.unicode.sort_by_key(|u| u.0);   // `note` looks names up by code point
        s
    }

    /// Layout 150: (text, note) for a query, best first, at most `limit`. `tones`: the skin-tone
    /// order for emoji that have them (names from [`TONES`]; ones not listed aren't offered).
    pub fn symbols(&self, query: &str, tones: &[String], limit: usize) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        let push = |out: &mut Vec<(String, String)>, text: &str, note: &str| {
            if out.len() < limit && !out.iter().any(|o| o.0 == text) {
                out.push((text.to_string(), note.to_string()));
            }
        };
        let q = query.trim();
        let mut chars = q.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            if let Some(list) = self.by_key.get(&c) {
                for &i in list {
                    push(&mut out, &self.symbols[i].text, &self.symbols[i].name);
                }
            } else if c.is_ascii_alphanumeric() {
                for (text, note) in letter_forms(c) {
                    push(&mut out, &text, note);
                }
            }
            return out;
        }
        let query: Vec<String> = words(q);
        let query: Vec<&str> = query.iter().map(String::as_str).collect();
        // Name matches before tag matches; an exact name first of all.
        let mut found: Vec<(u8, usize, &Item)> = Vec::new();
        for (n, item) in self.symbols.iter().chain(self.emoji.iter()).enumerate() {
            if !matches(&query, &item.words) {
                continue;
            }
            let rank = if item.name.eq_ignore_ascii_case(q) {
                0
            } else if matches(&query, &words(&item.name)) {
                1
            } else {
                2
            };
            found.push((rank, n, item));
        }
        found.sort_by_key(|f| (f.0, f.1));
        for (_, _, item) in found {
            if item.tones.len() == 5 {
                for tone in tones {
                    match TONES.iter().position(|t| t == tone) {
                        Some(0) => push(&mut out, &item.text, &item.name),
                        Some(i) => push(&mut out, &item.tones[i - 1], &format!("{}: {} skin tone", item.name, TONES[i])),
                        None => {}
                    }
                }
            } else {
                push(&mut out, &item.text, &item.name);
            }
            if out.len() >= limit {
                break;
            }
        }
        out
    }

    /// Layout 151: (text, note) for a query: a hex code's character first, then names, at most
    /// `limit`. Notes are `U+XXXX name`.
    pub fn unicode(&self, query: &str, limit: usize) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        let q = query.trim().to_lowercase();
        let hex = q.strip_prefix("u+").or_else(|| q.strip_prefix('u').filter(|h| h.starts_with(|c: char| c.is_ascii_digit()))).unwrap_or(&q);
        let by_code = (q.starts_with(|c: char| c.is_ascii_digit()) || q.starts_with('u'))
            && (1..=6).contains(&hex.len())
            && hex.chars().all(|c| c.is_ascii_hexdigit());
        if by_code {
            if let Some(c) = u32::from_str_radix(hex, 16).ok().and_then(char::from_u32) {
                out.push((c.to_string(), self.note(c as u32)));
            }
        }
        let query: Vec<String> = words(&q);
        let query: Vec<&str> = query.iter().map(String::as_str).collect();
        if query.is_empty() || (q.chars().count() < 2 && !by_code) {
            return out;
        }
        let mut found: Vec<(u8, usize, u32)> = Vec::new();
        for (cp, name) in &self.unicode {
            // Cheap first: every query word must occur somewhere in the name.
            if !query.iter().all(|q| name.contains(q)) {
                continue;
            }
            let w = words(name);
            if !matches(&query, &w) {
                continue;
            }
            let rank = if *name == q { 0 } else { 1 };
            found.push((rank, name.len(), *cp));
        }
        found.sort();
        for (_, _, cp) in found {
            if out.len() >= limit {
                break;
            }
            if let Some(c) = char::from_u32(cp) {
                if !out.iter().any(|o| o.0 == c.to_string()) {
                    out.push((c.to_string(), self.note(cp)));
                }
            }
        }
        out
    }

    fn note(&self, cp: u32) -> String {
        let name = self
            .unicode
            .binary_search_by_key(&cp, |u| u.0)
            .map(|i| self.unicode[i].1.as_str())
            .unwrap_or("");
        format!("U+{cp:04X} {name}").trim_end().to_string()
    }
}

/// A letter's or digit's other forms: full-width, circled, accented (letters), super- and
/// subscript and roman numerals (digits).
fn letter_forms(c: char) -> Vec<(String, &'static str)> {
    let mut out = Vec::new();
    let lower = c.to_ascii_lowercase();
    let wide = char::from_u32(c as u32 + 0xFEE0).map(|w| (w.to_string(), "fullwidth"));
    if c.is_ascii_digit() {
        let d = c as u32 - '0' as u32;
        out.extend(wide);
        let circled = if d == 0 { '⓪' } else { char::from_u32(0x245F + d).unwrap_or(c) };
        out.push((circled.to_string(), "circled"));
        out.push((["⁰", "¹", "²", "³", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹"][d as usize].to_string(), "superscript"));
        out.push((char::from_u32(0x2080 + d).unwrap_or(c).to_string(), "subscript"));
        if d >= 1 {
            out.push((char::from_u32(0x215F + d).unwrap_or(c).to_string(), "roman numeral"));
        }
    } else {
        for upper in [false, true] {
            if let Some(vs) = crate::hold::variants(lower, upper) {
                out.extend(vs.iter().map(|v| (v.to_string(), "variant")));
            }
        }
        out.extend(char::from_u32(lower as u32 + 0xFEE0).map(|w| (w.to_string(), "fullwidth")));
        out.push((char::from_u32(0x24D0 + lower as u32 - 'a' as u32).unwrap_or(c).to_string(), "circled"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Search {
        Search::parse(
            "$\tusd\tdollar money cash currency bill\t¥ $\n€\teur\teuro money\t¥\n。\tideographic full stop\tperiod kuten\t.\n…\tellipsis\tdots\t.\n",
            "😀\tgrinning face\tface|grin|happy|smile\t\n👋\twaving hand\thand|wave\t👋🏻 👋🏼 👋🏽 👋🏾 👋🏿\n",
            "0041\tLATIN CAPITAL LETTER A\n1F600\tGRINNING FACE\n00B7\tMIDDLE DOT\n",
        )
    }
    fn tones(t: &[&str]) -> Vec<String> {
        t.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn symbols_by_key_name_and_tag() {
        let s = s();
        let all = tones(&TONES);
        let texts = |q: &str| s.symbols(q, &all, 20).into_iter().map(|x| x.0).collect::<Vec<_>>();
        assert_eq!(texts("."), ["。", "…"]);
        assert_eq!(texts("¥"), ["$", "€"]);
        assert_eq!(texts("usd"), ["$"]);
        assert_eq!(texts("money"), ["$", "€"]);
        assert_eq!(texts("happy face"), ["😀"]);
        assert_eq!(s.symbols("usd", &all, 5)[0].1, "usd");
        assert!(texts("1").contains(&"①".to_string()));
    }

    #[test]
    fn skin_tones_follow_the_setting() {
        let s = s();
        let got: Vec<String> = s.symbols("wave", &tones(&["medium", "none"]), 10).into_iter().map(|x| x.0).collect();
        assert_eq!(got, ["👋🏽", "👋"]);
    }

    #[test]
    fn unicode_by_code_and_name() {
        let s = s();
        assert_eq!(s.unicode("1f600", 5)[0], ("😀".to_string(), "U+1F600 grinning face".to_string()));
        assert_eq!(s.unicode("u+41", 5)[0].0, "A");
        assert_eq!(s.unicode("middle dot", 5)[0].0, "·");
        assert_eq!(s.unicode("face", 5)[0].0, "😀");   // `face` is hex too, but starts with a letter: a name
    }
}
