//! The user dictionary (docs/dictionary-contract.md): positive words, negative words and 초성
//! shortcuts, from `dictionary.json`. One per process, shared by every engine; the shell loads it.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::Deserialize;

use crate::hangul::{choseong, decompose, is_syllable};
use crate::layout::Token;

/// How hard a negative word is pushed down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Strength {
    /// Ranked well below what it would be; still offered.
    #[default]
    Soft,
    /// Never offered.
    Strong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Korean,
    English,
    Japanese,
}

#[derive(Deserialize)]
struct PositiveIn {
    word: String,
    #[serde(default)]
    language: Language,
    count: Option<u32>,
    confidence: Option<String>,
}

#[derive(Deserialize)]
struct NegativeIn {
    word: String,
    #[serde(default)]
    language: Language,
    #[serde(default)]
    strength: Strength,
}

#[derive(Deserialize)]
struct ShortcutIn {
    choseong: String,
    word: String,
}

#[derive(Deserialize)]
struct FileIn {
    format: String,
    version: u32,
    #[serde(default)]
    positive: Vec<PositiveIn>,
    #[serde(default)]
    negative: Vec<NegativeIn>,
    #[serde(default)]
    shortcuts: Vec<ShortcutIn>,
}

/// Limits (docs/dictionary-contract.md): entries beyond them are ignored.
pub const MAX_POSITIVE: usize = 20_000;
pub const MAX_NEGATIVE: usize = 20_000;
pub const MAX_SHORTCUTS: usize = 5_000;

#[derive(Debug, Default)]
pub struct UserDict {
    positive: HashMap<(Language, String), u32>,
    negative: HashMap<(Language, String), Strength>,
    /// 초성 string → words, in file order.
    shortcuts: HashMap<String, Vec<String>>,
    /// Positive Korean words by their 초성 string, for 초성 input.
    korean_by_choseong: HashMap<String, Vec<(String, u32)>>,
}

/// What `parse` found, for the shell to report.
#[derive(Debug, Default, serde::Serialize)]
pub struct Summary {
    pub ok: bool,
    pub error: String,
    pub positive: usize,
    pub negative: usize,
    pub shortcuts: usize,
}

fn count_of(count: Option<u32>, confidence: Option<&str>) -> u32 {
    count.unwrap_or(match confidence {
        Some("confirmed") => 200,
        Some("likely") => 50,
        Some("tentative") => 10,
        _ => 50,
    })
}

impl UserDict {
    /// Parses `dictionary.json`. On an error, an empty dictionary and the reason.
    pub fn parse(json: &str) -> (UserDict, Summary) {
        let file: FileIn = match serde_json::from_str(json) {
            Ok(f) => f,
            Err(e) => return (UserDict::default(), Summary { error: e.to_string(), ..Summary::default() }),
        };
        if file.format != "bada-dictionary" || file.version != 1 {
            let error = format!("format must be \"bada-dictionary\", version 1 (got {:?}, {})", file.format, file.version);
            return (UserDict::default(), Summary { error, ..Summary::default() });
        }
        let mut d = UserDict::default();
        for p in file.positive.into_iter().take(MAX_POSITIVE) {
            let word = p.word.trim().to_string();
            if word.is_empty() {
                continue;
            }
            let word = if p.language == Language::English { word.to_lowercase() } else { word };
            let count = count_of(p.count, p.confidence.as_deref());
            if p.language == Language::Korean && word.chars().all(is_syllable) {
                d.korean_by_choseong.entry(choseong(&word)).or_default().push((word.clone(), count));
            }
            *d.positive.entry((p.language, word)).or_insert(0) += count;
        }
        for list in d.korean_by_choseong.values_mut() {
            list.sort_by(|a, b| b.1.cmp(&a.1));
        }
        for n in file.negative.into_iter().take(MAX_NEGATIVE) {
            let word = n.word.trim().to_string();
            if word.is_empty() {
                continue;
            }
            let word = if n.language == Language::English { word.to_lowercase() } else { word };
            d.negative.insert((n.language, word), n.strength);
        }
        for s in file.shortcuts.into_iter().take(MAX_SHORTCUTS) {
            let key: String = s.choseong.chars().filter(|c| !c.is_whitespace()).collect();
            if !key.is_empty() && !s.word.trim().is_empty() {
                d.shortcuts.entry(key).or_default().push(s.word.trim().to_string());
            }
        }
        let summary = Summary {
            ok: true,
            error: String::new(),
            positive: d.positive.len(),
            negative: d.negative.len(),
            shortcuts: d.shortcuts.values().map(Vec::len).sum(),
        };
        (d, summary)
    }

    pub fn count(&self, language: Language, word: &str) -> u32 {
        let key = if language == Language::English { word.to_lowercase() } else { word.to_string() };
        self.positive.get(&(language, key)).copied().unwrap_or(0)
    }

    pub fn negative(&self, language: Language, word: &str) -> Option<Strength> {
        let key = if language == Language::English { word.to_lowercase() } else { word.to_string() };
        self.negative.get(&(language, key)).copied()
    }

    /// Positive Korean words for 초성 tokens (same rule as the built-in list).
    pub fn choseong_words(&self, tokens: &[Token]) -> Vec<(String, u32)> {
        let key: String = tokens.iter().map(|t| t.ini).collect();
        self.korean_by_choseong
            .get(&key)
            .map(|list| {
                list.iter()
                    .filter(|(w, _)| w.chars().zip(tokens).all(|(c, t)| !t.a || decompose(c).is_some_and(|d| d.1 == 'ㅏ')))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The user's own 초성 shortcuts for exactly this 초성 string.
    pub fn shortcuts(&self, choseong: &str) -> &[String] {
        self.shortcuts.get(choseong).map_or(&[], Vec::as_slice)
    }
}

static CURRENT: RwLock<Option<Arc<UserDict>>> = RwLock::new(None);

/// Replaces the process's dictionary (an empty string clears it). Returns what was loaded.
pub fn load(json: &str) -> Summary {
    if json.trim().is_empty() {
        *CURRENT.write().unwrap_or_else(|e| e.into_inner()) = None;
        return Summary { ok: true, ..Summary::default() };
    }
    let (dict, summary) = UserDict::parse(json);
    *CURRENT.write().unwrap_or_else(|e| e.into_inner()) = if summary.ok { Some(Arc::new(dict)) } else { None };
    summary
}

/// The process's dictionary, if any.
pub fn current() -> Option<Arc<UserDict>> {
    CURRENT.read().unwrap_or_else(|e| e.into_inner()).clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_contract() {
        let (d, s) = UserDict::parse(r#"{
            "format": "bada-dictionary", "version": 1,
            "positive": [{"word": "데이터센터", "count": 30}, {"word": "Kubernetes", "language": "english"},
                         {"word": "サーバー", "language": "japanese", "confidence": "confirmed"}],
            "negative": [{"word": "감소합니다", "strength": "strong"}, {"word": "god", "language": "english"}],
            "shortcuts": [{"choseong": "ㄱㅅ", "word": "감사합니다"}],
            "unknown_field": true
        }"#);
        assert!(s.ok, "{}", s.error);
        assert_eq!((s.positive, s.negative, s.shortcuts), (3, 2, 1));
        assert_eq!(d.count(Language::Korean, "데이터센터"), 30);
        assert_eq!(d.count(Language::English, "kubernetes"), 50);
        assert_eq!(d.count(Language::Japanese, "サーバー"), 200);
        assert_eq!(d.negative(Language::Korean, "감소합니다"), Some(Strength::Strong));
        assert_eq!(d.negative(Language::English, "God"), Some(Strength::Soft));
        assert_eq!(d.shortcuts("ㄱㅅ"), ["감사합니다"]);
        let t: Vec<Token> = "ㄷㅇㅌㅅㅌ".chars().map(|c| Token { ini: c, a: false }).collect();
        assert_eq!(d.choseong_words(&t)[0].0, "데이터센터");
        let (_, bad) = UserDict::parse(r#"{"format": "other", "version": 1}"#);
        assert!(!bad.ok);
    }
}
