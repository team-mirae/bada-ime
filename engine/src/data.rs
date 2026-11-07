//! The engine's data, loaded once by the shell and shared by every engine: the Korean word list
//! (with its 초성 index), the English word list, the hanja dictionary and the OECD menu.

use std::collections::{HashMap, HashSet};

use crate::hangul::{choseong, decompose, is_syllable};
use crate::hanja::Dictionary;
use crate::layout::Token;

/// How many words each 초성 string keeps, most frequent first.
const PER_CHOSEONG: usize = 40;

#[derive(Default)]
pub struct Data {
    korean: HashMap<String, u32>,
    by_choseong: HashMap<String, Vec<(String, u32)>>,
    english: HashSet<String>,
    japanese: HashSet<String>,
    pub hanja: Dictionary,
    /// Kana-kanji conversion (`kanji.bin`); None without the file.
    pub kanji: Option<crate::kanji::Kanji>,
    /// (hanja, Korean name)
    pub oecd: Vec<(String, String)>,
}

impl Data {
    /// From the files in `engine/data`: `ko-words.tsv`, `en-words.txt`, `hanja.tsv`, `oecd.tsv`.
    /// Any of them may be empty. `with_japanese` adds `ja-words.txt`.
    pub fn parse(korean: &str, english: &str, hanja: &str, oecd: &str) -> Data {
        let mut data = Data {
            hanja: Dictionary::parse(hanja),
            ..Data::default()
        };
        for line in korean.lines().filter(|l| !l.starts_with('#')) {
            let mut f = line.split('\t');
            let (Some(word), Some(count)) = (f.next(), f.next()) else {
                continue;
            };
            let Ok(count) = count.trim().parse::<u32>() else {
                continue;
            };
            data.korean.insert(word.to_string(), count);
            if word.chars().all(is_syllable) {
                data.by_choseong
                    .entry(choseong(word))
                    .or_default()
                    .push((word.to_string(), count));
            }
        }
        for list in data.by_choseong.values_mut() {
            list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            list.truncate(PER_CHOSEONG);
        }
        for line in english.lines().filter(|l| !l.starts_with('#')) {
            let w = line.trim();
            if !w.is_empty() {
                data.english.insert(w.to_lowercase());
            }
        }
        for line in oecd.lines().filter(|l| !l.starts_with('#')) {
            let mut f = line.split('\t');
            if let (Some(h), Some(name)) = (f.next(), f.next()) {
                data.oecd.push((h.to_string(), name.to_string()));
            }
        }
        data
    }

    /// Adds the Japanese word list (`ja-words.txt`: one word per line, `word<TAB>count` allowed).
    pub fn with_japanese(mut self, japanese: &str) -> Data {
        for line in japanese.lines().filter(|l| !l.starts_with('#')) {
            if let Some(w) = line
                .split('\t')
                .next()
                .map(str::trim)
                .filter(|w| !w.is_empty())
            {
                self.japanese.insert(w.to_string());
            }
        }
        self
    }

    /// Adds kana-kanji conversion (`kanji.bin`). A malformed file is ignored (no kanji).
    pub fn with_kanji(mut self, data: &[u8]) -> Data {
        self.kanji = if data.is_empty() { None } else { crate::kanji::Kanji::parse(data).ok() };
        self
    }

    /// Whether a kana string is a known Japanese word or ends in a known word (Japanese has no spaces).
    pub fn is_japanese(&self, kana: &str) -> bool {
        self.japanese.contains(kana)
    }

    /// How often a Korean word was seen (0: never).
    pub fn korean_count(&self, word: &str) -> u32 {
        self.korean.get(word).copied().unwrap_or(0)
    }

    pub fn is_english(&self, word: &str) -> bool {
        self.english.contains(&word.to_lowercase())
    }

    /// Words for 초성 tokens (§15): same syllable count, each syllable's initial matching, and ㅏ
    /// where a token carries it (the final left open, §15.1). Most frequent first.
    pub fn choseong_words(&self, tokens: &[Token], limit: usize) -> Vec<(String, u32)> {
        let key: String = tokens.iter().map(|t| t.ini).collect();
        let Some(list) = self.by_choseong.get(&key) else {
            return Vec::new();
        };
        list.iter()
            .filter(|(w, _)| {
                w.chars()
                    .zip(tokens)
                    .all(|(c, t)| !t.a || decompose(c).is_some_and(|d| d.1 == 'ㅏ'))
            })
            .take(limit)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choseong_lookup() {
        let d = Data::parse(
            "안녕하세요\t50\n안녕히세요\t1\n아니하세요\t3\nㅋㅋ\t9\n",
            "hello\n",
            "",
            "美\t미국\n",
        );
        let t = |s: &str, a: &[bool]| {
            s.chars()
                .zip(a)
                .map(|(c, &a)| Token { ini: c, a })
                .collect::<Vec<_>>()
        };
        let all = d.choseong_words(&t("ㅇㄴㅎㅅㅇ", &[false; 5]), 9);
        assert_eq!(all[0].0, "안녕하세요");
        assert_eq!(all.len(), 3);
        let with_a = d.choseong_words(&t("ㅇㄴㅎㅅㅇ", &[false, false, true, false, false]), 9);
        assert_eq!(
            with_a.iter().map(|w| w.0.as_str()).collect::<Vec<_>>(),
            ["안녕하세요", "아니하세요"]
        );
        assert!(d.is_english("Hello"));
        assert_eq!(d.korean_count("ㅋㅋ"), 9);
        assert_eq!(d.oecd[0].0, "美");
    }
}
