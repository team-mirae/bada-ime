//! The engine's data, loaded once by the shell and shared by every engine: the Korean word list
//! (with its 초성 index), the English word list, the hanja dictionary and the OECD menu.

use std::collections::HashSet;

use crate::hangul::{decompose, is_syllable};
use crate::hanja::Dictionary;
use crate::layout::Token;

/// The Korean word list, packed: words sorted by bytes in one string, with offsets and counts,
/// and the word ids again in 초성 order (most frequent first within one 초성 string). About 30 bytes
/// a word, against ~300 in hash maps: a million words fit in some 30 MB.
#[derive(Default)]
struct KoreanWords {
    text: String,
    /// Word `i` is `text[starts[i]..starts[i + 1]]`.
    starts: Vec<u32>,
    counts: Vec<u32>,
    /// Ids of the all-syllable words, sorted by their 초성 string, then by count (descending).
    by_choseong: Vec<u32>,
}

/// A 초성 string as one number: its length (up to 24) in the top bits, then 5 bits per initial, so
/// equal strings get equal keys and keys sort by length, then initials.
fn pack(initials: impl Iterator<Item = char>) -> u128 {
    let mut key: u128 = 0;
    let mut len: u128 = 0;
    for c in initials.take(24) {
        let i = crate::hangul::INITIALS.iter().position(|&x| x == c).map_or(31, |i| i as u128 + 1);
        key = (key << 5) | i;
        len += 1;
    }
    (len << 120) | key
}

impl KoreanWords {
    /// From `ko-words.tsv` lines. The file is written sorted by word bytes (build_ko_words.py), so
    /// this reads it straight in; an unsorted file is sorted first (more memory while loading).
    fn build(tsv: &str) -> KoreanWords {
        let parse = |line: &str| -> Option<(String, u32)> {
            let mut f = line.split('\t');
            let word = f.next()?;
            let count = f.next()?.trim().parse::<u32>().ok()?;
            (!word.is_empty()).then(|| (word.to_string(), count))
        };
        let lines = || tsv.lines().filter(|l| !l.starts_with('#'));
        let mut k = KoreanWords { text: String::with_capacity(tsv.len()), ..KoreanWords::default() };
        let mut sorted = true;
        let mut last = String::new();
        for line in lines() {
            let Some((w, c)) = parse(line) else { continue };
            if w.as_str() < last.as_str() {
                sorted = false;
                break;
            }
            if w == last {
                continue;
            }
            k.starts.push(k.text.len() as u32);
            k.text.push_str(&w);
            k.counts.push(c);
            last = w;
        }
        if !sorted {
            let mut all: Vec<(String, u32)> = lines().filter_map(parse).collect();
            all.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            all.dedup_by(|b, a| a.0 == b.0);
            k = KoreanWords { text: String::new(), ..KoreanWords::default() };
            for (w, c) in &all {
                k.starts.push(k.text.len() as u32);
                k.text.push_str(w);
                k.counts.push(*c);
            }
        }
        k.starts.push(k.text.len() as u32);
        k.text.shrink_to_fit();
        // The 초성 order: ids only, keys computed while sorting (no string or key per word kept).
        let keys: Vec<u128> = (0..k.counts.len() as u32)
            .map(|i| if k.word(i).chars().all(is_syllable) { pack(k.word(i).chars().map(|c| decompose(c).map_or('ㅇ', |d| d.0))) } else { 0 })
            .collect();
        let mut ids: Vec<u32> = (0..k.counts.len() as u32).filter(|&i| keys[i as usize] != 0).collect();
        ids.sort_unstable_by(|&a, &b| {
            keys[a as usize].cmp(&keys[b as usize]).then(k.counts[b as usize].cmp(&k.counts[a as usize])).then(a.cmp(&b))
        });
        drop(keys);
        k.by_choseong = ids;
        k
    }

    fn word(&self, i: u32) -> &str {
        &self.text[self.starts[i as usize] as usize..self.starts[i as usize + 1] as usize]
    }

    fn count(&self, word: &str) -> u32 {
        let n = self.counts.len() as u32;
        let (mut lo, mut hi) = (0u32, n);
        while lo < hi {
            let mid = (lo + hi) / 2;
            match self.word(mid).cmp(word) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => return self.counts[mid as usize],
            }
        }
        0
    }

    /// The ids whose 초성 string is `key`, most frequent first. (Words over 24 syllables share keys
    /// by their first 24; the caller checks the length.)
    fn with_choseong(&self, key: &str) -> &[u32] {
        let want = pack(key.chars());
        let packed = |i: &u32| pack(self.word(*i).chars().map(|c| decompose(c).map_or('ㅇ', |d| d.0)));
        let start = self.by_choseong.partition_point(|i| packed(i) < want);
        let end = start + self.by_choseong[start..].partition_point(|i| packed(i) == want);
        &self.by_choseong[start..end]
    }
}

/// Japanese words usually written in kana (tools/build_kana_native.py), built in: small, and the
/// shell needn't ship another file.
const KANA_NATIVE: &str = include_str!("../data/ja-kana-native.txt");

#[derive(Default)]
pub struct Data {
    korean: KoreanWords,
    english: HashSet<String>,
    japanese: HashSet<String>,
    /// Japanese words usually written in kana (`ja-kana-native.txt`, built in).
    kana_native: HashSet<String>,
    pub hanja: Dictionary,
    /// Kana-kanji conversion (`kanji.bin`); None without the file.
    pub kanji: Option<crate::kanji::Kanji>,
    /// The jamo 4-gram model for typo suggestions (`ko-jamo-4gram.bin`); None without the file.
    pub lm: Option<crate::typo::JamoLm>,
    /// Symbols, emoji and character names for layouts 150 and 151.
    pub search: crate::search::Search,
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
        data.korean = KoreanWords::build(korean);
        data.kana_native = KANA_NATIVE
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| l.split('\t').next())
            .map(String::from)
            .collect();
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

    /// Adds the searches of layouts 150 and 151 (`symbols.tsv`, `emoji.tsv`, `unicode-names.tsv`).
    pub fn with_search(mut self, symbols: &str, emoji: &str, unicode: &str) -> Data {
        self.search = crate::search::Search::parse(symbols, emoji, unicode);
        self
    }

    /// Adds the typo model (`ko-jamo-4gram.bin`). A malformed or missing file means no typo suggestions.
    pub fn with_lm(mut self, data: &[u8]) -> Data {
        self.lm = crate::typo::JamoLm::parse(data);
        self
    }

    /// Adds kana-kanji conversion (`kanji.bin`). A malformed file is ignored (no kanji).
    pub fn with_kanji(mut self, data: &[u8]) -> Data {
        self.kanji = if data.is_empty() { None } else { crate::kanji::Kanji::parse(data).ok() };
        if let Some(k) = &mut self.kanji {
            k.prefer_kana(self.kana_native.iter().map(String::as_str));
        }
        self
    }

    /// Whether a kana string is a Japanese word usually written in kana (この, あなた).
    pub fn is_kana_native(&self, kana: &str) -> bool {
        self.kana_native.contains(kana)
    }

    /// Whether a kana string is a known Japanese word or ends in a known word (Japanese has no spaces).
    pub fn is_japanese(&self, kana: &str) -> bool {
        self.japanese.contains(kana)
    }

    /// How often a Korean word was seen (0: never).
    pub fn korean_count(&self, word: &str) -> u32 {
        self.korean.count(word)
    }

    pub fn is_english(&self, word: &str) -> bool {
        self.english.contains(&word.to_lowercase())
    }

    /// Words for 초성 tokens (unused at present, bada-layout.md §8): same syllable count, each
    /// syllable's initial matching, and ㅏ where a token carries it (the final left open). Most
    /// frequent first.
    pub fn choseong_words(&self, tokens: &[Token], limit: usize) -> Vec<(String, u32)> {
        let key: String = tokens.iter().map(|t| t.ini).collect();
        self.korean
            .with_choseong(&key)
            .iter()
            .map(|&i| (self.korean.word(i), self.korean.counts[i as usize]))
            .filter(|(w, _)| w.chars().count() == tokens.len())
            .filter(|(w, _)| w.chars().zip(tokens).all(|(c, t)| !t.a || decompose(c).is_some_and(|d| d.1 == 'ㅏ')))
            .take(limit)
            .map(|(w, c)| (w.to_string(), c))
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
