//! The engine: muhenkan and henkan modes (spec §4), the signed suggestion menu (§6.4), the keys
//! (§6), F-key actions (§6.5) and Bada's actions (bada-layout.md §6).
//!
//! Event in → output out. The shell owns input-source switching (英数, かな, HYPER/MEH+かな) and
//! the host text; the engine says what to commit, what to show and whether it took the key.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::data::Data;
use crate::japanese;
use crate::key::{Key, Mods};
use crate::layout::{self, BADA, K3S_JIS, KANA, KANA_KOREAN, KOREAN_ROMAN, KTS, LEGACY_KOREAN, LEGACY_MOONSHOT, Press, QWERTY, ROMAJI, SYMBOLS, UNICODE};
use crate::userdict::{Language, Strength, UserDict};
use crate::width::{Width, convert};

/// The user's settings (spec §5.3, §6.4, §6.5), as the shell stores them.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    /// Henkan candidates, in the user's order (earlier wins ties): profile `mac-kana`'s own list.
    pub enabled: Vec<u32>,
    /// Henkan profiles (§6.6): `mac-kana` (かな), `henkan` (変換), `katakana` (カタカナ). A profile
    /// missing here: `mac-kana` uses `enabled`, the others link to `mac-kana`.
    pub profiles: BTreeMap<String, Profile>,
    /// Layout id → weight (§5.3): every entry that layout raises has its score multiplied by it.
    /// Missing: 1. Clamped to 0–10.
    pub weights: HashMap<u32, f64>,
    pub default_muhenkan: u32,
    /// Negative menu entries (§6.4: −1 hiragana, −2 katakana for 20 and 21). Off by default.
    pub script_forms: bool,
    /// Emoji skin tones in the order layout 150 offers them ("none", "light", "medium-light",
    /// "medium", "medium-dark", "dark"); a tone left out isn't offered.
    pub skin_tones: Vec<String>,
    /// The one method that contributes negative menu entries (§6.4). None: the lowest enabled id
    /// that offers them.
    pub negative_method: Option<u32>,
    /// Intercept F6–F10 at all (§6.5).
    pub intercept_f: bool,
    /// "F6" … "F10" → "swallow" | "pass" | "<method>.<action>". Missing: swallow.
    pub fkeys: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            enabled: vec![BADA],
            profiles: BTreeMap::new(),
            weights: HashMap::new(),
            default_muhenkan: QWERTY,
            script_forms: false,
            skin_tones: crate::search::TONES.iter().map(|t| t.to_string()).collect(),
            negative_method: None,
            intercept_f: true,
            fkeys: HashMap::new(),
        }
    }
}

/// Results a search layout (150, 151) lists, at most.
const SEARCH_RESULTS: usize = 40;

/// Typo suggestions listed under layout 13's reading, at most.
const TYPO_SUGGESTIONS: usize = 3;
/// Japanese: rows given to a reading's most common words (`ja-common-words.txt`).
const COMMON_ROWS: usize = 3;

/// A henkan profile (§6.6): its own candidates, or a link to another profile.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct Profile {
    /// Another profile whose candidates this one uses ("mac-kana", "henkan", "katakana").
    pub link: Option<String>,
    /// This profile's own candidates, in order (unused while it links).
    pub enabled: Vec<u32>,
    /// This profile's own F keys (§6.5); None: `mac-kana`'s (the settings' `intercept_f`, `fkeys`).
    pub f_keys: Option<FKeys>,
}

/// F6–F10 for one profile (§6.5): intercepted or not, and each key's binding.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct FKeys {
    pub intercept: bool,
    /// "F6" … "F10" → "swallow" | "pass" | "<method>.<action>". Missing: swallow.
    pub keys: HashMap<String, String>,
}

/// The henkan profiles (§6.6), in the order the settings show them.
pub const PROFILES: [&str; 3] = ["mac-kana", "henkan", "katakana"];

/// The F-key actions each method publishes (§6.5): (method id, action id, label).
pub const ACTIONS: &[(u32, &str, &str)] = &[
    (
        BADA,
        "raw",
        "Raw jamo: every key as its jamo, nothing composed",
    ),
    (BADA, "prediction-1", "Use the 1st prediction"),
    (BADA, "prediction-2", "Use the 2nd prediction"),
    (BADA, "prediction-3", "Use the 3rd prediction"),
    (BADA, "prediction-4", "Use the 4th prediction"),
    (BADA, "prediction-5", "Use the 5th prediction"),
    (BADA, "qwerty-fullwidth", "QWERTY, full-width"),
    (BADA, "qwerty-halfwidth", "QWERTY, half-width"),
    (
        BADA,
        "hanja",
        "Hanja: selection, else staged text, else OECD countries",
    ),
    (LEGACY_KOREAN, "commit", "Commit as typed (this layout's reading, no ranking)"),
    (LEGACY_KOREAN, "commit-bare", "Commit as typed, implied ㅇ dropped (가어 → 가ㅓ)"),
    (LEGACY_MOONSHOT, "commit", "Commit as typed (this layout's reading, no ranking)"),
    (LEGACY_MOONSHOT, "commit-bare", "Commit as typed, implied ㅇ dropped (가어 → 가ㅓ)"),
    (KANA, "hiragana", "Hiragana"),
    (KANA, "katakana", "Katakana"),
    (KANA, "halfwidth-katakana", "Half-width katakana"),
    (
        KANA,
        "fullwidth-latin",
        "Full-width Latin (the keys as QWERTY)",
    ),
    (
        KANA,
        "halfwidth-latin",
        "Half-width Latin (the keys as QWERTY)",
    ),
    (ROMAJI, "hiragana", "Hiragana"),
    (ROMAJI, "katakana", "Katakana"),
    (ROMAJI, "halfwidth-katakana", "Half-width katakana"),
    (
        ROMAJI,
        "fullwidth-latin",
        "Full-width Latin (the letters typed)",
    ),
    (
        ROMAJI,
        "halfwidth-latin",
        "Half-width Latin (the letters typed)",
    ),
];

/// Methods that offer script forms as negative menu entries (§6.4).
const SCRIPT_FORM_METHODS: &[u32] = &[KANA, ROMAJI];

/// A Japanese method's kana reading of the presses.
fn kana_reading(method: u32, presses: &[Press]) -> String {
    if method == KANA {
        japanese::kana_compose(presses)
    } else {
        japanese::romaji_compose(presses)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Muhenkan,
    Henkan,
}

/// What the engine asks the shell to do with one event.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Output {
    /// Text to insert (replacing the marked text, or the selection if `replace_selection`).
    pub commit: String,
    pub replace_selection: bool,
    /// The marked text after the commit.
    pub preedit: String,
    /// The menu's rows, top to bottom (negative entries first); empty: no menu.
    pub menu: Vec<String>,
    /// The highlighted row, and the row of entry 0.
    pub selected: usize,
    pub zero: usize,
    /// The engine took the key; false: the app gets it (after the commit).
    pub consumed: bool,
    /// The menu is one row of single letters (⌥ + letter variants); `selected` past its end: none.
    pub horizontal: bool,
    /// Per menu row: a note shown in gray at the right (romaji, a kana reading), and whether the
    /// row is Japanese (a Japanese font). Empty when no row has one.
    pub notes: Vec<String>,
    pub japanese: Vec<bool>,
    /// The engine needs the host's selection to finish: "hanja", "full" or "half". The shell calls
    /// `host_selection` with it.
    pub need_selection: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Override {
    Raw,
    Qwerty(Width),
    /// A Japanese method's reading in a script: hiragana, katakana, or half-width katakana.
    Kana {
        method: u32,
        script: KanaScript,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KanaScript {
    Hiragana,
    Katakana,
    HalfKatakana,
}

fn in_script(hiragana: &str, script: KanaScript) -> String {
    match script {
        KanaScript::Hiragana => hiragana.to_string(),
        KanaScript::Katakana => japanese::katakana(hiragana),
        KanaScript::HalfKatakana => convert(&japanese::katakana(hiragana), Width::Half),
    }
}

#[derive(Clone, Debug, Default)]
struct Entry {
    text: String,
    score: f64,
    /// Shown in gray at the row's right: romaji for kana, the kana reading for kanji.
    note: String,
    /// A Japanese reading: drawn in a Japanese font.
    japanese: bool,
    /// Where it came from, for the 초성 diagnostics (spec §8).
    origin: Origin,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Origin {
    #[default]
    Other,
    /// Layout `id`'s composition of the presses.
    Composed(u32),
    /// A 초성 prediction of layout `id`.
    Choseong(u32),
}

impl Entry {
    fn new(text: String, score: f64) -> Entry {
        Entry { text, score, ..Entry::default() }
    }

    /// A kana entry, its romaji as the note.
    fn kana(text: String, score: f64) -> Entry {
        let note = crate::romaji::romaji(&text).unwrap_or_default();
        Entry { text, score, note, japanese: true, origin: Origin::Other }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PickTarget {
    /// Replace the staged text: commit `before` + the pick.
    Staged { before: String },
    /// Replace the host's selection: `before` + the pick.
    Selection { before: String },
    /// Insert at the caret.
    Insert,
}

#[derive(Clone, Debug)]
struct Pick {
    rows: Vec<(String, String)>,
    index: usize,
    target: PickTarget,
}

pub struct Engine {
    data: Arc<Data>,
    settings: Settings,
    mode: Mode,
    /// Text frozen in the preedit (半角/全角 applied to what showed), before the presses.
    prefix: String,
    presses: Vec<Press>,
    over: Option<Override>,
    entries: Vec<Entry>,
    negatives: Vec<Entry>,
    /// Signed index into the menu (§6.4): 0 is the top entry, negatives are script forms.
    index: i32,
    pick: Option<Pick>,
    /// Waiting for the host's selection for this action.
    waiting: Option<String>,
    /// Muhenkan's open ⌥ + letter variants list.
    held: Option<crate::hold::Held>,
    /// A dictionary pinned to this engine (tests); None: the process's (`userdict::current`).
    pinned: Option<Option<std::sync::Arc<UserDict>>>,
    /// The henkan profile in use (§6.6).
    profile: String,
}

impl Engine {
    pub fn new(data: Arc<Data>) -> Engine {
        Engine {
            data,
            settings: Settings::default(),
            mode: Mode::Henkan,
            prefix: String::new(),
            presses: Vec::new(),
            over: None,
            entries: Vec::new(),
            negatives: Vec::new(),
            index: 0,
            pick: None,
            waiting: None,
            held: None,
            pinned: None,
            profile: PROFILES[0].to_string(),
        }
    }

    pub fn set_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.settings.enabled.retain(|&id| layout::is_known(id));
        for p in self.settings.profiles.values_mut() {
            p.enabled.retain(|&id| layout::is_known(id));
        }
        self.recompute();
    }

    /// Switches henkan profile (§6.6): what's staged is committed first. Unknown names: `mac-kana`.
    pub fn set_profile(&mut self, name: &str) -> Output {
        let name = if PROFILES.contains(&name) { name } else { PROFILES[0] };
        let mut out = Output::default();
        if name != self.profile {
            out.commit = self.take_shown();
            self.profile = name.to_string();
            self.recompute();
        }
        self.finish(out)
    }

    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Whether every candidate in use is a search layout (150, 151): Space then searches.
    fn searching(&self) -> bool {
        let c = self.candidates(&self.profile);
        !c.is_empty() && c.iter().all(|&id| layout::is_search(id))
    }

    /// The candidates of a profile, links followed (a loop or a dead end: `mac-kana`'s own list).
    pub fn candidates(&self, name: &str) -> Vec<u32> {
        let mut at = name.to_string();
        for _ in 0..PROFILES.len() {
            let link = match self.settings.profiles.get(&at) {
                Some(Profile { link: Some(l), .. }) => l.clone(),
                Some(Profile { link: None, enabled, .. }) if at != PROFILES[0] => return enabled.clone(),
                None if at != PROFILES[0] => PROFILES[0].to_string(),
                _ => return self.settings.enabled.clone(),
            };
            if !PROFILES.contains(&link.as_str()) {
                break;
            }
            at = link;
        }
        self.settings.enabled.clone()
    }

    /// Pins a user dictionary to this engine instead of the process's (None in it: no dictionary).
    pub fn pin_dictionary(&mut self, dict: Option<std::sync::Arc<UserDict>>) {
        self.pinned = Some(dict);
        self.recompute();
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Switching mode commits what's staged (the shell inserts the returned text).
    pub fn set_mode(&mut self, mode: Mode) -> Output {
        let mut out = Output::default();
        if mode != self.mode {
            out.commit = match self.held.take() {
                Some(h) => h.shown().to_string(),
                None => self.take_shown(),
            };
        }
        self.mode = mode;
        self.finish(out)
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Commits whatever shows (focus lost, mode switch, a click).
    pub fn flush(&mut self) -> Output {
        let mut out = Output::default();
        if let Some(h) = self.held.take() {
            out.commit = h.shown().to_string();
            return self.finish(out);
        }
        out.commit = match self.pick.take() {
            Some(p) => match p.target {
                PickTarget::Staged { .. } => self.take_shown(),
                _ => String::new(),
            },
            None => self.take_shown(),
        };
        self.finish(out)
    }

    fn staged(&self) -> bool {
        !self.prefix.is_empty() || !self.presses.is_empty()
    }

    /// The text that shows: the prefix and the entry at the index.
    fn shown(&self) -> String {
        let entry = if self.index < 0 {
            self.negatives.get((-self.index - 1) as usize)
        } else {
            self.entries.get(self.index as usize)
        };
        format!("{}{}", self.prefix, entry.map_or("", |e| e.text.as_str()))
    }

    fn clear(&mut self) {
        self.prefix.clear();
        self.presses.clear();
        self.over = None;
        self.entries.clear();
        self.negatives.clear();
        self.index = 0;
        self.pick = None;
    }

    fn take_shown(&mut self) -> String {
        let text = self.shown();
        self.record();
        self.clear();
        text
    }

    /// The 초성 diagnostics (spec §8) for what's about to be committed: henkan mode only, and only
    /// when the whole text is the entry (nothing frozen before it, no override).
    fn record(&self) {
        if self.mode != Mode::Henkan || !self.prefix.is_empty() || self.over.is_some() || self.index < 0 {
            return;
        }
        let Some(entry) = self.entries.get(self.index as usize) else { return };
        match entry.origin {
            Origin::Composed(id) => crate::diag::typed(id, &self.presses, &layout::details(id, &self.presses)),
            Origin::Choseong(id) => crate::diag::choseong(id, &self.presses, &entry.text),
            Origin::Other => {}
        }
    }

    // ---------- readings ----------

    /// How often a Korean word is seen: the built-in count plus the user dictionary's.
    fn ko_count(&self, user: &Option<std::sync::Arc<UserDict>>, word: &str) -> u32 {
        self.data.korean_count(word) + user.as_ref().map_or(0, |u| u.count(Language::Korean, word))
    }

    fn recompute(&mut self) {
        self.entries.clear();
        self.negatives.clear();
        self.index = 0;
        if self.presses.is_empty() {
            return;
        }
        if let Some(over) = self.over {
            let text = match over {
                Override::Raw => layout::raw(BADA, &self.presses),
                Override::Qwerty(w) => convert(&layout::qwerty(&self.presses), w),
                Override::Kana { method, script } => {
                    in_script(&kana_reading(method, &self.presses), script)
                }
            };
            let entry = match over {
                Override::Kana { .. } => Entry::kana(text, 0.0),
                _ => Entry::new(text, 0.0),
            };
            self.entries.push(entry);
            return;
        }
        let enabled = self.candidates(&self.profile);
        let mut entries: Vec<Entry> = Vec::new();
        let user = match &self.pinned {
            Some(pinned) => pinned.clone(),
            None => crate::userdict::current(),
        };
        for (rank, &id) in enabled.iter().enumerate() {
            // Earlier in the user's order wins ties.
            let tie = -(rank as f64) * 0.001;
            if !self.presses.iter().all(|&p| layout::takes(id, p)) {
                continue;
            }
            let start = entries.len();
            match id {
                QWERTY => {
                    let text = layout::qwerty(&self.presses);
                    let letters = text.chars().all(|c| c.is_ascii_alphabetic());
                    let score = if self.data.is_english(&text) || user.as_ref().is_some_and(|u| u.count(Language::English, &text) > 0) {
                        90.0
                    } else if letters {
                        30.0
                    } else {
                        20.0
                    };
                    entries.push(Entry {
                        text,
                        score: score + tie,
                        ..Entry::default()
                    });
                }
                KANA_KOREAN | KOREAN_ROMAN => {
                    // The table's reading shows; every other pronunciation that spells a known word is
                    // offered under it, most frequent first (docs/kana-korean.md, "Readings").
                    let all = if id == KANA_KOREAN {
                        crate::kana_korean::readings(&self.presses, 4096)
                    } else {
                        crate::korean_roman::readings(&self.presses, 4096)
                    };
                    let Some(default) = all.first().cloned() else {
                        continue;
                    };
                    let count = self.ko_count(&user, &default);
                    let loose = default
                        .chars()
                        .any(|c| ('\u{3131}'..='\u{3163}').contains(&c));
                    let score = if count > 0 {
                        100.0 + (count as f64).ln()
                    } else if loose {
                        40.0
                    } else {
                        80.0
                    };
                    entries.push(Entry {
                        text: default,
                        score: score + tie,
                        ..Entry::default()
                    });
                    let mut known: Vec<(String, u32)> = all[1..]
                        .iter()
                        .map(|r| (r.clone(), self.ko_count(&user, r)))
                        .filter(|r| r.1 > 0)
                        .collect();
                    known.sort_by(|a, b| b.1.cmp(&a.1));
                    for (i, (text, _)) in known.into_iter().take(8).enumerate() {
                        entries.push(Entry {
                            text,
                            score: score + tie - 0.01 * (i + 1) as f64,
                        ..Entry::default()
                    });
                    }
                }
                KTS | LEGACY_KOREAN | LEGACY_MOONSHOT | BADA | K3S_JIS => {
                    if let Some(tokens) = layout::choseong_tokens(id, &self.presses) {
                        // What shows is the normal composition: a consonant after a ⇧ token is its
                        // final (⇧ㄱ ㄴ → 간), consonants alone stay as they are (ㅇㄴ). The 초성
                        // predictions are listed with it.
                        let literal = layout::compose_presses(id, &self.presses);
                        let attached = literal != layout::tokens_text(&tokens);
                        let count = self.ko_count(&user, &literal);
                        let loose = literal.chars().any(|c| ('\u{3131}'..='\u{3163}').contains(&c));
                        let score = if tokens.len() == 1 {
                            200.0
                        } else if attached && count > 0 {
                            // A real word (⇧ㄱ ㄴ → 간) beats the 초성 guesses.
                            120.0 + (count as f64).ln()
                        } else if attached && !loose {
                            99.0
                        } else if attached {
                            // Loose jamo left (ㅇㄴ핫ㅇ): the 초성 reading is likelier.
                            45.0
                        } else if count > 0 {
                            101.0 + (count as f64).ln()
                        } else {
                            50.0
                        };
                        entries.push(Entry {
                            text: literal,
                            score: score + tie,
                            origin: Origin::Composed(id),
                            ..Entry::default()
                    });
                        if tokens.len() >= 2 {
                            // The user's own: 초성 shortcuts first, then positive words.
                            if let Some(u) = &user {
                                let typed: String = tokens.iter().map(|t| t.ini).collect();
                                for (i, word) in u.shortcuts(&typed).iter().enumerate() {
                                    entries.push(Entry {
                                        text: word.clone(),
                                        score: 160.0 - 0.01 * i as f64 + tie,
                                        origin: Origin::Choseong(id),
                                        ..Entry::default()
                                    });
                                }
                                for (word, count) in u.choseong_words(&tokens).into_iter().take(9) {
                                    let built = self.data.korean_count(&word);
                                    entries.push(Entry {
                                        text: word,
                                        score: 97.0 + ((count + built) as f64).ln() + tie,
                                        origin: Origin::Choseong(id),
                                        ..Entry::default()
                                    });
                                }
                            }
                            for (word, count) in self.data.choseong_words(&tokens, 9) {
                                let count = count + user.as_ref().map_or(0, |u| u.count(Language::Korean, &word));
                                entries.push(Entry {
                                    text: word,
                                    score: 95.0 + (count as f64).ln() + tie,
                                    origin: Origin::Choseong(id),
                                    ..Entry::default()
                    });
                            }
                        }
                    } else {
                        let text = layout::compose_presses(id, &self.presses);
                        let count = self.ko_count(&user, &text);
                        let loose = text.chars().any(|c| ('\u{3131}'..='\u{3163}').contains(&c));
                        let score = if count > 0 {
                            100.0 + (count as f64).ln()
                        } else if loose {
                            40.0
                        } else {
                            80.0
                        };
                        entries.push(Entry {
                            text,
                            score: score + tie,
                            origin: Origin::Composed(id),
                            ..Entry::default()
                    });
                    }
                }
                SYMBOLS | UNICODE => {
                    // The search as typed first; what it finds under it, with names as notes.
                    let query = layout::query(&self.presses);
                    let found = if id == SYMBOLS {
                        self.data.search.symbols(&query, &self.settings.skin_tones, SEARCH_RESULTS)
                    } else {
                        self.data.search.unicode(&query, SEARCH_RESULTS)
                    };
                    let own = found.iter().find(|f| f.0 == query).map(|f| f.1.clone()).unwrap_or_default();
                    entries.push(Entry { text: query, score: 60.0 + tie, note: own, ..Entry::default() });
                    for (i, (text, note)) in found.into_iter().enumerate() {
                        entries.push(Entry { text, score: 59.0 + tie - 0.01 * i as f64, note, ..Entry::default() });
                    }
                }
                KANA | ROMAJI => {
                    let text = kana_reading(id, &self.presses);
                    let loose = text.contains(japanese::DAKUTEN)
                        || text.contains(japanese::HANDAKUTEN)
                        || text.chars().any(|c| c.is_ascii_alphabetic());
                    let known = self.data.is_japanese(&text)
                        || self.data.is_kana_native(&text)
                        || user.as_ref().is_some_and(|u| u.count(Language::Japanese, &text) > 0);
                    let score = if known {
                        92.0
                    } else if loose {
                        30.0
                    } else {
                        70.0
                    };
                    // Kanji conversions (docs/japanese.md), each with the kana reading as its note,
                    // above the kana itself. The kana goes where conversion ranks it (kana-native
                    // words convert to kana: この, ちょっと), a known word's no lower than second
                    // (あなた under 貴方), a kana-native word first.
                    let mut kana_score = score + tie;
                    if !loose {
                        if let Some(kanji) = &self.data.kanji {
                            let mut row = 0;
                            for (conv, _) in kanji.convert(&text, 6) {
                                let at = score + tie + 0.5 - 0.01 * row as f64;
                                if conv == text {
                                    kana_score = kana_score.max(at + 0.001);
                                    continue;
                                }
                                // Its katakana is a row already when script forms are on (−2).
                                if self.settings.script_forms && conv == japanese::katakana(&text) {
                                    continue;
                                }
                                entries.push(Entry {
                                    text: conv,
                                    score: at,
                                    note: text.clone(),
                                    japanese: true,
                                    origin: Origin::Other,
                                });
                                row += 1;
                            }
                            // The most common words of the reading (一 for いち, 死 and 詩 for し), right
                            // under the first conversion: conversion of a lone word ranks them low.
                            for (i, word) in self.data.common_japanese(&text).iter().take(COMMON_ROWS).enumerate() {
                                entries.push(Entry {
                                    text: word.clone(),
                                    score: score + tie + 0.5 - 0.004 - 0.0001 * i as f64,
                                    note: text.clone(),
                                    japanese: true,
                                    origin: Origin::Other,
                                });
                            }
                            if known {
                                kana_score = kana_score.max(score + tie + 0.5 - 0.005);
                            }
                            if self.data.is_kana_native(&text) {
                                kana_score = kana_score.max(score + tie + 0.6);
                            }
                        }
                    }
                    entries.push(Entry::kana(text, kana_score));
                }
                _ => {}
            }
            // The layout's weight (§5.3), on everything it raised.
            let weight = self.settings.weights.get(&id).copied().filter(|w| w.is_finite()).map_or(1.0, |w| w.clamp(0.0, 10.0));
            if weight != 1.0 {
                entries[start..].iter_mut().for_each(|e| e.score *= weight);
            }
            // The user's negative words (docs/dictionary-contract.md): soft ones far down, strong
            // ones gone.
            if let Some(u) = &user {
                let language = match id {
                    QWERTY => Language::English,
                    KANA | ROMAJI => Language::Japanese,
                    _ => Language::Korean,
                };
                let mut i = start;
                while i < entries.len() {
                    match u.negative(language, &entries[i].text) {
                        Some(Strength::Strong) => {
                            entries.remove(i);
                            continue;
                        }
                        Some(Strength::Soft) => entries[i].score -= 60.0,
                        None => {}
                    }
                    i += 1;
                }
            }
        }
        // Script forms (§6.4), when on: one method, at most two entries: −1 hiragana, −2 katakana.
        let method = self
            .settings
            .negative_method
            .filter(|_| self.settings.script_forms)
            .filter(|m| enabled.contains(m) && SCRIPT_FORM_METHODS.contains(m))
            .or_else(|| {
                if !self.settings.script_forms {
                    return None;
                }
                enabled
                    .iter()
                    .copied()
                    .filter(|m| SCRIPT_FORM_METHODS.contains(m))
                    .min()
            });
        if let Some(method) = method.filter(|&m| self.presses.iter().all(|&p| layout::takes(m, p)))
        {
            let hira = kana_reading(method, &self.presses);
            self.negatives = vec![Entry::kana(hira.clone(), 0.0), Entry::kana(japanese::katakana(&hira), 0.0)];
        }
        // One row per text, at its best score.
        entries.sort_by(|a, b| b.score.total_cmp(&a.score));
        let mut seen = std::collections::HashSet::new();
        entries.retain(|e| seen.insert(e.text.clone()));
        // Typo suggestions (bada-layout.md §5): right under layout 13's reading, never above it
        // and never first.
        if let Some(lm) = self.data.lm.as_ref().filter(|_| enabled.contains(&BADA)) {
            if let Some(at) = entries.iter().position(|e| e.origin == Origin::Composed(BADA)) {
                let mut n = 0;
                for (text, _) in crate::typo::suggest(lm, &self.presses, TYPO_SUGGESTIONS) {
                    let unwanted = user.as_ref().is_some_and(|u| u.negative(Language::Korean, &text) == Some(Strength::Strong));
                    if unwanted || entries.iter().any(|e| e.text == text) {
                        continue;
                    }
                    n += 1;
                    let score = entries[at].score - 0.001 * n as f64;
                    entries.insert(at + n, Entry { text, score, note: "typo?".into(), ..Entry::default() });
                }
            }
        }
        self.entries = entries;
    }

    fn finish(&mut self, mut out: Output) -> Output {
        if let Some(h) = &self.held {
            out.preedit = h.shown().to_string();
            out.menu = h.variants.iter().map(|c| c.to_string()).collect();
            out.selected = h.selected.unwrap_or(h.variants.len());
            out.horizontal = true;
            return out;
        }
        if let Some(pick) = &self.pick {
            out.preedit = match &pick.target {
                PickTarget::Staged { before } => format!("{before}{}", pick.rows[pick.index].0),
                _ => String::new(),
            };
            out.menu = pick
                .rows
                .iter()
                .map(|(h, c)| {
                    if c.is_empty() {
                        h.clone()
                    } else {
                        format!("{h}  {c}")
                    }
                })
                .collect();
            out.selected = pick.index;
            out.zero = 0;
            return out;
        }
        out.preedit = self.shown();
        let rows = self.negatives.len() + self.entries.len();
        if self.staged() && rows > 1 {
            let rows: Vec<&Entry> = self.negatives.iter().rev().chain(self.entries.iter()).collect();
            out.menu = rows.iter().map(|e| e.text.clone()).collect();
            if rows.iter().any(|e| !e.note.is_empty() || e.japanese) {
                out.notes = rows.iter().map(|e| e.note.clone()).collect();
                out.japanese = rows.iter().map(|e| e.japanese).collect();
            }
            out.zero = self.negatives.len();
            out.selected = (out.zero as i32 + self.index) as usize;
        }
        out
    }

    fn step(&mut self, forward: bool) {
        let low = -(self.negatives.len() as i32);
        let high = self.entries.len() as i32 - 1;
        // No wrap: stop at the ends (§6.4).
        self.index = if forward {
            (self.index + 1).min(high)
        } else {
            (self.index - 1).max(low)
        };
    }

    // ---------- keys ----------

    /// One key event.
    pub fn key(&mut self, key: Key, mods: Mods) -> Output {
        let out = match self.mode {
            Mode::Muhenkan => self.key_muhenkan(key, mods),
            Mode::Henkan => self.key_henkan(key, mods),
        };
        self.finish(out)
    }

    fn key_muhenkan(&mut self, key: Key, mods: Mods) -> Output {
        let mut out = Output::default();
        if self.held.is_some() {
            if let Some(done) = self.key_held(key, mods) {
                return done;
            }
        }
        if mods.is_shortcut() {
            return out;
        }
        // ⌥ + a letter with variants: the letter, its variants beside it, none chosen.
        if mods.alt && key.is_letter() {
            let letter = layout::jis_char(key, false).unwrap_or('?');
            if let Some(variants) = crate::hold::variants(letter, mods.shift) {
                let letter = if mods.shift { letter.to_ascii_uppercase() } else { letter };
                self.held = Some(crate::hold::Held { key, letter, variants, selected: None });
                out.consumed = true;
                return out;
            }
        }
        match key {
            Key::Space if mods.alt => {
                out.commit = "_".into();
                out.consumed = true;
            }
            Key::Backquote if !mods.alt => {
                out.need_selection = Some(if mods.shift { "half" } else { "full" }.into());
                self.waiting = out.need_selection.clone();
                out.consumed = true;
            }
            _ => {}
        }
        out
    }

    /// Keys while a variants list is open (jis-korean spec §3.6.3). None: not handled here (the
    /// list was closed by committing what showed; the key goes on as usual).
    fn key_held(&mut self, key: Key, mods: Mods) -> Option<Output> {
        let h = self.held.as_mut()?;
        let n = h.variants.len();
        let consumed = |commit: String| Output { commit, consumed: true, ..Output::default() };
        // The same ⌥ + letter again: the next variant, wrapping.
        if mods.alt && !mods.is_shortcut() && key == h.key {
            h.selected = Some(h.selected.map_or(0, |i| (i + 1) % n));
            return Some(consumed(String::new()));
        }
        if !mods.alt && !mods.is_shortcut() && !mods.shift {
            if let Some(d) = key.code().strip_prefix("Digit").and_then(|d| d.parse::<usize>().ok()) {
                let index = if d == 0 { 9 } else { d - 1 };
                let held = self.held.take()?;
                if index < n {
                    return Some(consumed(held.variants[index].to_string()));
                }
                // Past the list: the letter, and the digit goes on to the app.
                return Some(Output { commit: held.letter.to_string(), ..Output::default() });
            }
        }
        match key {
            Key::ArrowRight | Key::ArrowDown => {
                h.selected = Some(h.selected.map_or(0, |i| (i + 1) % n));
                Some(consumed(String::new()))
            }
            Key::ArrowLeft | Key::ArrowUp => {
                h.selected = Some(h.selected.map_or(n - 1, |i| (i + n - 1) % n));
                Some(consumed(String::new()))
            }
            Key::Enter => {
                let held = self.held.take()?;
                Some(consumed(held.shown().to_string()))
            }
            Key::Escape | Key::Backspace => {
                self.held = None;
                Some(consumed(String::new()))
            }
            _ => {
                // Any other key keeps what shows; then it's handled as usual (another ⌥ + letter
                // opens its own list).
                let held = self.held.take()?;
                let mut rest = self.key_muhenkan(key, mods);
                rest.commit = format!("{}{}", held.shown(), rest.commit);
                Some(rest)
            }
        }
    }

    fn commit_then_pass(&mut self) -> Output {
        Output {
            commit: self.take_shown(),
            ..Output::default()
        }
    }

    fn key_henkan(&mut self, key: Key, mods: Mods) -> Output {
        if self.pick.is_some() {
            if let Some(out) = self.key_pick(key, mods) {
                return out;
            }
        }
        if let Some(f) = key.f_index() {
            return self.f_key(f, mods);
        }
        if mods.is_shortcut() {
            return self.commit_then_pass();
        }
        let staged = self.staged();
        let consumed = |commit: String| Output {
            commit,
            consumed: true,
            ..Output::default()
        };
        match key {
            Key::Lang1 => {
                if !self.presses.is_empty() {
                    self.step(!mods.shift);
                }
                return consumed(String::new());
            }
            Key::Tab | Key::ArrowDown | Key::ArrowUp if staged && !self.presses.is_empty() => {
                let back = key == Key::ArrowUp || (key == Key::Tab && mods.shift);
                self.step(!back);
                return consumed(String::new());
            }
            Key::Enter if mods.shift && staged => return consumed(self.take_shown()),
            // Only search layouts (150, 151) in use: Space is part of the search, ⇧Space a space.
            Key::Space if !mods.alt && self.searching() => {
                if mods.shift {
                    return consumed(format!("{} ", self.take_shown()));
                }
                self.presses.push(Press { key, shift: false, alt: false });
                self.recompute();
                return consumed(String::new());
            }
            Key::Space if mods.alt => {
                let text = format!("{}\u{3000}", self.take_shown());
                return consumed(text);
            }
            Key::Backquote if !mods.alt => {
                let w = if mods.shift { Width::Half } else { Width::Full };
                if staged {
                    self.prefix = convert(&self.shown(), w);
                    self.presses.clear();
                    self.over = None;
                    self.recompute();
                    return consumed(String::new());
                }
                let mut out = consumed(String::new());
                out.need_selection = Some(if mods.shift { "half" } else { "full" }.into());
                self.waiting = out.need_selection.clone();
                return out;
            }
            Key::Backspace if staged => {
                if self.presses.pop().is_none() {
                    self.prefix.pop();
                }
                if self.presses.is_empty() {
                    self.over = None;
                }
                self.recompute();
                return consumed(String::new());
            }
            _ => {}
        }
        if key.is_kana_space() && !(mods.alt && !mods.shift) {
            let press = Press {
                key,
                shift: mods.shift,
                alt: mods.alt,
            };
            let any = self.over.is_some() || self.candidates(&self.profile).iter().any(|&id| layout::takes(id, press));
            if any {
                self.presses.push(press);
                self.recompute();
                return consumed(String::new());
            }
        }
        self.commit_then_pass()
    }

    /// Keys while a hanja (or OECD) list is open. None: close it by choosing and handle the key.
    fn key_pick(&mut self, key: Key, mods: Mods) -> Option<Output> {
        let consumed = Output {
            consumed: true,
            ..Output::default()
        };
        let pick = self.pick.as_mut()?;
        match key {
            Key::ArrowDown | Key::Lang1 | Key::Tab if !(mods.shift && key != Key::ArrowDown) => {
                pick.index = (pick.index + 1).min(pick.rows.len() - 1);
                Some(consumed)
            }
            Key::ArrowUp | Key::Lang1 | Key::Tab => {
                pick.index = pick.index.saturating_sub(1);
                Some(consumed)
            }
            Key::Escape | Key::Backspace => {
                self.pick = None;
                Some(consumed)
            }
            Key::Enter | Key::Space => Some(self.choose_pick()),
            _ => {
                let mut out = self.choose_pick();
                let rest = self.key_henkan(key, mods);
                out.commit.push_str(&rest.commit);
                out.consumed = rest.consumed;
                out.need_selection = rest.need_selection;
                Some(out)
            }
        }
    }

    fn choose_pick(&mut self) -> Output {
        let Some(pick) = self.pick.take() else {
            return Output::default();
        };
        let chosen = pick.rows[pick.index].0.clone();
        let mut out = Output {
            consumed: true,
            ..Output::default()
        };
        match pick.target {
            PickTarget::Staged { before } => {
                self.clear();
                out.commit = before + &chosen;
            }
            PickTarget::Selection { before } => {
                out.commit = before + &chosen;
                out.replace_selection = true;
            }
            PickTarget::Insert => out.commit = chosen,
        }
        out
    }

    /// F6–F10 (§6.5).
    fn f_key(&mut self, f: u8, mods: Mods) -> Output {
        let pass = Output::default();
        let swallow = Output {
            consumed: true,
            ..Output::default()
        };
        // The profile's own F keys, else mac-kana's (§6.6).
        let own = self.settings.profiles.get(&self.profile).and_then(|p| p.f_keys.as_ref());
        let (intercept, keys) = match own {
            Some(f) if self.profile != PROFILES[0] => (f.intercept, &f.keys),
            _ => (self.settings.intercept_f, &self.settings.fkeys),
        };
        if !intercept {
            return pass;
        }
        let binding = keys.get(&format!("F{f}")).cloned().unwrap_or_else(|| "swallow".into());
        match binding.as_str() {
            "pass" => return pass,
            "swallow" => return swallow,
            _ => {}
        }
        // Modified F keys: up to the bound method; Bada defines none, so they're swallowed.
        if mods.shift || mods.alt || mods.ctrl || mods.meta {
            return swallow;
        }
        let Some((method, action)) = binding.split_once('.') else {
            return swallow;
        };
        let Ok(method) = method.parse::<u32>() else {
            return swallow;
        };
        if !self.candidates(&self.profile).contains(&method)
            || !ACTIONS.iter().any(|a| a.0 == method && a.1 == action)
        {
            return swallow;
        }
        self.action(method, action)
    }

    /// A method's action (bada-layout.md §6; Japanese: docs/japanese.md).
    pub fn action(&mut self, method: u32, action: &str) -> Output {
        let mut out = Output {
            consumed: true,
            ..Output::default()
        };
        if method == LEGACY_KOREAN || method == LEGACY_MOONSHOT {
            if !self.presses.is_empty() && (action == "commit" || action == "commit-bare") {
                let text = if action == "commit" {
                    layout::compose_presses(method, &self.presses)
                } else {
                    layout::compose_bare(method, &self.presses)
                };
                out.commit = format!("{}{}", self.prefix, text);
                self.clear();
            }
            return out;
        }
        if SCRIPT_FORM_METHODS.contains(&method) {
            if !self.presses.is_empty() {
                self.over = Some(match action {
                    "hiragana" => Override::Kana {
                        method,
                        script: KanaScript::Hiragana,
                    },
                    "katakana" => Override::Kana {
                        method,
                        script: KanaScript::Katakana,
                    },
                    "halfwidth-katakana" => Override::Kana {
                        method,
                        script: KanaScript::HalfKatakana,
                    },
                    "fullwidth-latin" => Override::Qwerty(Width::Full),
                    "halfwidth-latin" => Override::Qwerty(Width::Half),
                    _ => return out,
                });
                self.recompute();
            }
            return out;
        }
        if method != BADA {
            return out;
        }
        if let Some(n) = action
            .strip_prefix("prediction-")
            .and_then(|n| n.parse::<usize>().ok())
        {
            if n >= 1 && n <= self.entries.len() && self.over.is_none() {
                self.index = (n - 1) as i32;
                out.commit = self.take_shown();
            }
            return out;
        }
        match action {
            "raw" if !self.presses.is_empty() => {
                self.over = Some(Override::Raw);
                self.recompute();
            }
            "qwerty-fullwidth" if !self.presses.is_empty() => {
                self.over = Some(Override::Qwerty(Width::Full));
                self.recompute();
            }
            "qwerty-halfwidth" if !self.presses.is_empty() => {
                self.over = Some(Override::Qwerty(Width::Half));
                self.recompute();
            }
            "hanja" => {
                if self.staged() {
                    let text = self.shown();
                    if let Some(pick) = self.hanja_pick(&text, true) {
                        self.pick = Some(pick);
                    }
                } else {
                    out.need_selection = Some("hanja".into());
                    self.waiting = out.need_selection.clone();
                }
            }
            _ => {}
        }
        out
    }

    fn hanja_pick(&self, text: &str, staged: bool) -> Option<Pick> {
        let whole = self.data.hanja.lookup(text);
        let (reading, candidates) = if !whole.is_empty() {
            (text, whole)
        } else {
            self.data.hanja.longest_suffix(text)?
        };
        let before = text[..text.len() - reading.len()].to_string();
        let rows = candidates
            .iter()
            .map(|c| (c.hanja.clone(), c.comment.clone()))
            .collect::<Vec<_>>();
        let target = if staged {
            PickTarget::Staged { before }
        } else {
            PickTarget::Selection { before }
        };
        Some(Pick {
            rows,
            index: 0,
            target,
        })
    }

    /// The host's selection, after an output asked for it (`need_selection`).
    pub fn host_selection(&mut self, selected: &str) -> Output {
        let mut out = Output {
            consumed: true,
            ..Output::default()
        };
        let waiting = self.waiting.take();
        match waiting.as_deref() {
            Some(w @ ("full" | "half")) if !selected.is_empty() => {
                out.commit = convert(
                    selected,
                    if w == "half" {
                        Width::Half
                    } else {
                        Width::Full
                    },
                );
                out.replace_selection = true;
            }
            Some("hanja") => {
                if !selected.is_empty() {
                    if let Some(pick) = self.hanja_pick(selected, false) {
                        self.pick = Some(pick);
                    }
                } else if !self.data.oecd.is_empty() {
                    let rows = self.data.oecd.clone();
                    self.pick = Some(Pick {
                        rows,
                        index: 0,
                        target: PickTarget::Insert,
                    });
                }
            }
            _ => {}
        }
        self.finish(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Key::*;

    /// A test engine, with no user dictionary whatever the process has loaded.
    fn engine() -> Engine {
        let data = Data::parse(
            "안녕하세요\t50\n안녕\t40\n한국\t60\n항구\t5\n아니하세요\t3\n한\t30\nㅋㅋ\t90\n감사합니다\t70\n학교\t20\n",
            "hello\nhan\n",
            "학교\t學校\t\n학\t學\t배울 학\n",
            "美\t미국\n英\t영국\n",
        );
        let mut e = Engine::new(Arc::new(data));
        e.pin_dictionary(None);
        e
    }
    fn press(e: &mut Engine, k: Key) -> Output {
        e.key(k, Mods::NONE)
    }
    fn shift(e: &mut Engine, k: Key) -> Output {
        e.key(k, Mods::SHIFT)
    }

    #[test]
    fn composes_and_commits_on_space() {
        let mut e = engine();
        // 한: ㅎ(V) ㅏ(H) ㄴ(K)
        press(&mut e, KeyV);
        press(&mut e, KeyH);
        let out = press(&mut e, KeyK);
        assert_eq!(out.preedit, "한");
        assert!(out.consumed);
        let out = press(&mut e, Space);
        assert_eq!(out.commit, "한");
        assert!(!out.consumed);
        assert_eq!(out.preedit, "");
    }

    #[test]
    fn shift_enter_commits_and_swallows() {
        let mut e = engine();
        press(&mut e, KeyV);
        press(&mut e, KeyH);
        let out = e.key(Enter, Mods::SHIFT);
        assert_eq!(out.commit, "하");
        assert!(out.consumed);
        // nothing staged: the app's
        assert!(!e.key(Enter, Mods::SHIFT).consumed);
    }

    #[test]
    fn literal_chat_jamo_win() {
        let mut e = engine();
        press(&mut e, KeyX);
        let out = press(&mut e, KeyX);
        assert_eq!(out.preedit, "ㅋㅋ");
    }

    #[test]
    fn menu_stops_at_the_ends() {
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![BADA, QWERTY];
        e.set_settings(s);
        for k in [KeyV, KeyH, KeyK] {
            press(&mut e, k);
        }
        let n = e.entries.len();
        assert!(n > 1);
        for _ in 0..n + 3 {
            press(&mut e, ArrowDown);
        }
        assert_eq!(e.index, n as i32 - 1);
        for _ in 0..n + 3 {
            press(&mut e, ArrowUp);
        }
        assert_eq!(e.index, 0, "no negative entries in this build");
    }

    #[test]
    fn f_keys_swallowed_by_default_and_bindable() {
        let mut e = engine();
        press(&mut e, KeyV);
        let out = press(&mut e, F6);
        assert!(out.consumed);
        assert_eq!(out.preedit, "ㅎ");
        let mut s = Settings::default();
        s.fkeys.insert("F7".into(), "pass".into());
        s.fkeys.insert("F8".into(), "13.raw".into());
        s.fkeys.insert("F9".into(), "13.qwerty-halfwidth".into());
        e.set_settings(s);
        assert!(!press(&mut e, F7).consumed);
        // raw: ㅎ ㅏ ㄴ stays as jamo
        let mut e2 = engine();
        e2.set_settings(e.settings().clone());
        for k in [KeyV, KeyH, KeyK] {
            press(&mut e2, k);
        }
        assert_eq!(press(&mut e2, F8).preedit, "ㅎㅏㄴ");
        let mut e3 = engine();
        e3.set_settings(e.settings().clone());
        for k in [KeyV, KeyH, KeyK] {
            press(&mut e3, k);
        }
        assert_eq!(press(&mut e3, F9).preedit, "vhk");
        let mut s = Settings::default();
        s.intercept_f = false;
        e3.set_settings(s);
        assert!(!press(&mut e3, F6).consumed);
    }

    #[test]
    fn prediction_actions_commit() {
        let mut e = engine();
        let mut s = Settings::default();
        s.fkeys.insert("F6".into(), "13.prediction-1".into());
        e.set_settings(s);
        // 안녕하세요: ㅇㅏㄴ ㄴㅕㅇ ㅎㅏ ㅅㅔ ㅇㅛ
        for k in [KeyF, KeyH, KeyK, KeyE, KeyU, KeyJ, KeyV, KeyH, KeyR, KeyO, KeyF, Digit9] {
            press(&mut e, k);
        }
        let out = press(&mut e, F6);
        assert_eq!(out.commit, "안녕하세요");
    }

    #[test]
    fn hanja_staged_selection_and_oecd() {
        let mut e = engine();
        let mut s = Settings::default();
        s.fkeys.insert("F7".into(), "13.hanja".into());
        e.set_settings(s);
        // 학교: ㅎ ㅏ ㄱ ㄱ ㅛ
        for k in [KeyV, KeyH, KeyD, KeyD, Digit9] {
            press(&mut e, k);
        }
        let out = press(&mut e, F7);
        assert_eq!(out.preedit, "學校");
        let out = press(&mut e, Enter);
        assert_eq!(out.commit, "學校");
        // nothing staged: asks for the selection
        let out = press(&mut e, F7);
        assert_eq!(out.need_selection.as_deref(), Some("hanja"));
        let out = e.host_selection("학");
        assert_eq!(out.menu[0], "學  배울 학");
        let out = press(&mut e, Enter);
        assert!(out.replace_selection);
        assert_eq!(out.commit, "學");
        // no selection: OECD
        press(&mut e, F7);
        let out = e.host_selection("");
        assert_eq!(out.menu, ["美  미국", "英  영국"]);
        press(&mut e, ArrowDown);
        assert_eq!(press(&mut e, Enter).commit, "英");
    }

    #[test]
    fn japanese_layouts_and_script_forms() {
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![ROMAJI];
        s.script_forms = true;
        s.fkeys.insert("F7".into(), "21.katakana".into());
        s.fkeys.insert("F8".into(), "21.halfwidth-katakana".into());
        e.set_settings(s);
        for k in [KeyR, KeyA, Minus, KeyM, KeyE, KeyN] {
            press(&mut e, k);
        }
        let out = press(&mut e, Tab);
        assert_eq!(out.preedit, "らーめん");
        // ⇧Tab from 0: −1 hiragana, −2 katakana; then it stops
        let mut e2 = engine();
        e2.set_settings(e.settings().clone());
        for k in [KeyR, KeyA, Minus, KeyM, KeyE, KeyN] {
            press(&mut e2, k);
        }
        assert_eq!(e2.key(Tab, Mods::SHIFT).preedit, "らーめん");
        let out = e2.key(Tab, Mods::SHIFT);
        assert_eq!(out.preedit, "ラーメン");
        assert_eq!(out.menu, ["ラーメン", "らーめん", "らーめん"]);
        assert_eq!(out.zero, 2);
        assert_eq!(e2.key(Tab, Mods::SHIFT).preedit, "ラーメン");
        // F-key actions
        let mut e3 = engine();
        e3.set_settings(e.settings().clone());
        for k in [KeyR, KeyA, Minus, KeyM, KeyE, KeyN] {
            press(&mut e3, k);
        }
        assert_eq!(press(&mut e3, F7).preedit, "ラーメン");
        assert_eq!(press(&mut e3, F8).preedit, "ﾗｰﾒﾝ");
        // Kana layout: か゛ → が
        let mut e4 = engine();
        let mut s = Settings::default();
        s.enabled = vec![KANA];
        e4.set_settings(s);
        press(&mut e4, KeyT);
        assert_eq!(press(&mut e4, BracketLeft).preedit, "が");
    }

    #[test]
    fn henkan_profiles_link_and_switch() {
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![QWERTY, KTS, KANA];
        s.profiles.insert("henkan".into(), Profile { link: None, enabled: vec![QWERTY, KTS, ROMAJI], f_keys: None });
        // katakana: not set, so linked to mac-kana
        e.set_settings(s.clone());
        assert_eq!(e.candidates("mac-kana"), [QWERTY, KTS, KANA]);
        assert_eq!(e.candidates("henkan"), [QWERTY, KTS, ROMAJI]);
        assert_eq!(e.candidates("katakana"), [QWERTY, KTS, KANA]);
        // a link to a link; a loop falls back to mac-kana's own list
        s.profiles.insert("katakana".into(), Profile { link: Some("henkan".into()), enabled: vec![], f_keys: None });
        e.set_settings(s.clone());
        assert_eq!(e.candidates("katakana"), [QWERTY, KTS, ROMAJI]);
        s.profiles.insert("henkan".into(), Profile { link: Some("katakana".into()), enabled: vec![], f_keys: None });
        e.set_settings(s.clone());
        assert_eq!(e.candidates("henkan"), [QWERTY, KTS, KANA]);
        // mac-kana itself can link
        s.profiles.insert("henkan".into(), Profile { link: None, enabled: vec![ROMAJI], f_keys: None });
        s.profiles.insert("mac-kana".into(), Profile { link: Some("henkan".into()), enabled: vec![], f_keys: None });
        e.set_settings(s);
        assert_eq!(e.candidates("mac-kana"), [ROMAJI]);
        // switching commits what's staged and reads with the new profile's layouts
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![KANA];
        s.profiles.insert("henkan".into(), Profile { link: None, enabled: vec![ROMAJI], f_keys: None });
        e.set_settings(s);
        press(&mut e, KeyT);
        assert_eq!(e.set_profile("henkan").commit, "か");
        assert_eq!(e.profile(), "henkan");
        press(&mut e, KeyK);
        assert_eq!(press(&mut e, KeyA).preedit, "か");
        assert_eq!(e.set_profile("nonsense").commit, "か");
        assert_eq!(e.profile(), "mac-kana");
    }

    #[test]
    fn layout_weights_scale_their_entries() {
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![QWERTY, BADA];
        e.set_settings(s.clone());
        // "han" in QWERTY is a known English word: it ranks above bada's reading of the same keys
        for k in [KeyH, KeyA, KeyN] {
            press(&mut e, k);
        }
        assert_eq!(e.entries[0].text, "han");
        // bada weighted up past it
        s.weights.insert(BADA, 3.0);   // its loose-jamo reading scores 40; ×3 beats the word's 90
        e.set_settings(s.clone());
        assert_ne!(e.entries[0].text, "han");
        // and QWERTY weighted to nothing ranks last
        s.weights.clear();
        s.weights.insert(QWERTY, 0.0);
        e.set_settings(s);
        assert_eq!(e.entries.last().unwrap().text, "han");
    }

    #[test]
    fn search_layouts_take_space_as_part_of_the_search() {
        let data = Data::parse("", "", "", "").with_search(
            "$\tusd\tdollar money cash currency bill\t¥ $\n",
            "😀\tgrinning face\tface|grin|happy|smile\t\n",
            "1F600\tGRINNING FACE\n",
        );
        let mut e = Engine::new(Arc::new(data));
        e.pin_dictionary(None);
        let mut s = Settings::default();
        s.enabled = vec![SYMBOLS];
        e.set_settings(s.clone());
        for k in [KeyH, KeyA, KeyP, KeyP, KeyY, Space, KeyF, KeyA, KeyC, KeyE] {
            press(&mut e, k);
        }
        let out = press(&mut e, Tab);
        assert_eq!(out.menu, ["happy face", "😀"]);
        assert_eq!(out.preedit, "😀");
        assert_eq!(out.notes[1], "grinning face");
        // ⇧Space: what shows, then a space
        assert_eq!(e.key(Space, Mods::SHIFT).commit, "😀 ");
        // usd → $; ¥ → its currencies
        for k in [KeyU, KeyS, KeyD] {
            press(&mut e, k);
        }
        assert_eq!(press(&mut e, Tab).preedit, "$");
        press(&mut e, Escape);
        // Unicode: a code
        s.enabled = vec![UNICODE];
        e.set_settings(s);
        for k in [Digit1, KeyF, Digit6, Digit0, Digit0] {
            press(&mut e, k);
        }
        assert_eq!(press(&mut e, Tab).preedit, "😀");
        // with another layout in use, Space commits as usual
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![SYMBOLS, QWERTY];
        e.set_settings(s);
        press(&mut e, KeyA);
        assert_eq!(press(&mut e, Space).commit, "a");
    }

    #[test]
    fn f_keys_per_profile() {
        let mut e = engine();
        let mut s = Settings::default();
        s.fkeys.insert("F8".into(), "13.raw".into());
        s.profiles.insert("henkan".into(), Profile { link: Some("mac-kana".into()), enabled: vec![], f_keys: Some(FKeys { intercept: false, keys: HashMap::new() }) });
        s.profiles.insert("katakana".into(), Profile { link: Some("mac-kana".into()), enabled: vec![], f_keys: None });
        e.set_settings(s);
        // mac-kana: F8 is raw
        for k in [KeyV, KeyH, KeyK] {
            press(&mut e, k);
        }
        assert_eq!(press(&mut e, F8).preedit, "ㅎㅏㄴ");
        // henkan: its own F keys, off: F8 goes to the app
        e.set_profile("henkan");
        press(&mut e, KeyV);
        assert!(!press(&mut e, F8).consumed);
        // katakana: none of its own, so mac-kana's
        e.set_profile("katakana");
        press(&mut e, KeyV);
        assert_eq!(press(&mut e, F8).preedit, "ㅎ");
    }

    #[test]
    fn script_forms_are_off_by_default() {
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![ROMAJI];
        e.set_settings(s);
        for k in [KeyR, KeyA, Minus, KeyM, KeyE, KeyN] {
            press(&mut e, k);
        }
        let out = e.key(Tab, Mods::SHIFT);
        assert_eq!(out.preedit, "らーめん");
        assert_eq!(out.zero, 0);
        assert!(e.negatives.is_empty());
    }

    #[test]
    fn kana_korean_shows_the_table_and_offers_words() {
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![KANA_KOREAN];
        e.set_settings(s);
        // は ん ぐ く: 한구쿠 shows; 한국 is offered under it
        for k in [KeyF, KeyY, KeyH, BracketLeft] {
            press(&mut e, k);
        }
        let out = press(&mut e, KeyH);
        assert_eq!(out.preedit, "한구쿠");
        assert_eq!(out.menu[..2], ["한구쿠", "한국"]);
        let out = press(&mut e, Tab);
        assert_eq!(out.preedit, "한국");
    }

    #[test]
    fn legacy_commit_actions() {
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![LEGACY_KOREAN, QWERTY];
        s.fkeys.insert("F6".into(), "11.commit-bare".into());
        s.fkeys.insert("F7".into(), "11.commit".into());
        e.set_settings(s);
        for k in [KeyR, KeyK, KeyJ] {
            press(&mut e, k);
        }
        let out = press(&mut e, F6);
        assert_eq!(out.commit, "가ㅓ");
        assert!(out.consumed);
        assert_eq!(out.preedit, "");
        // "commit" takes this layout's reading even when another ranks first (hello as 2-set)
        for k in [KeyH, KeyE, KeyL, KeyL, KeyO] {
            press(&mut e, k);
        }
        assert_eq!(press(&mut e, F7).commit, "옫이애");
    }

    #[test]
    fn notes_for_kana_rows() {
        let mut e = engine();
        let mut s = Settings::default();
        s.enabled = vec![ROMAJI];
        s.script_forms = true;
        e.set_settings(s);
        for k in [KeyR, KeyA, Minus, KeyM, KeyE, KeyN] {
            press(&mut e, k);
        }
        let out = press(&mut e, Tab);
        assert_eq!(out.notes, ["ra-men", "ra-men", "ra-men"]);
        assert!(out.japanese.iter().all(|&j| j));
    }

    #[test]
    fn choseong_diagnostics() {
        crate::diag::clear();
        crate::diag::set_enabled(true);
        let mut e = engine();
        // typed: 한 (ㅎ on V, ㄴ on K) then Space
        for k in [KeyV, KeyH, KeyK] {
            press(&mut e, k);
        }
        press(&mut e, Space);
        // muhenkan records nothing
        e.set_mode(Mode::Muhenkan);
        press(&mut e, KeyV);
        let json = crate::diag::export();
        let has = |frag: &str| assert!(json.contains(frag), "{frag} in {json}");
        has(r#""source":"typed","key":"KeyV","shift":false,"initial":"ㅎ","vowel":"ㅏ","final_":"ㄴ","final_key":"KeyK""#);
        assert!(!json.contains(r#""source":"choseong""#), "{json}");
        crate::diag::set_enabled(false);
    }

    #[test]
    fn letter_variants() {
        let alt = Mods { alt: true, ..Mods::NONE };
        let alt_shift = Mods { alt: true, shift: true, ..Mods::NONE };
        let mut e = engine();
        e.set_mode(Mode::Muhenkan);
        // ⌥u: u with its variants, none chosen; 2 → ü
        let out = e.key(KeyU, alt);
        assert_eq!(out.preedit, "u");
        assert_eq!(out.menu, ["û", "ü", "ù", "ú", "ū"]);
        assert!(out.horizontal);
        assert_eq!(out.selected, 5);
        assert_eq!(press(&mut e, Digit2).commit, "ü");
        // ⇧⌥u 2 → Ü; ⌥e ⌥e ⌥e Enter → é; ⌥e ← Enter → ę
        e.key(KeyU, alt_shift);
        assert_eq!(press(&mut e, Digit2).commit, "Ü");
        for _ in 0..3 {
            e.key(KeyE, alt);
        }
        assert_eq!(press(&mut e, Enter).commit, "é");
        e.key(KeyE, alt);
        press(&mut e, ArrowLeft);
        assert_eq!(press(&mut e, Enter).commit, "ę");
        // ⌥a b → a, then b goes to the app; ⌥o Esc → nothing; a digit past the list
        e.key(KeyA, alt);
        let out = press(&mut e, KeyB);
        assert_eq!(out.commit, "a");
        assert!(!out.consumed);
        e.key(KeyO, alt);
        let out = press(&mut e, Escape);
        assert_eq!((out.commit.as_str(), out.preedit.as_str()), ("", ""));
        e.key(KeyY, alt);
        let out = press(&mut e, Digit5);
        assert_eq!(out.commit, "y");
        assert!(!out.consumed);
        // ⌥ on a letter without variants: macOS's ⌥ layer (passed on)
        assert!(!e.key(KeyB, alt).consumed);
    }

    #[test]
    fn muhenkan_passes_through() {
        let mut e = engine();
        e.set_mode(Mode::Muhenkan);
        assert!(!press(&mut e, KeyA).consumed);
        assert!(!press(&mut e, F6).consumed);
        let out = e.key(
            Space,
            Mods {
                alt: true,
                ..Mods::NONE
            },
        );
        assert_eq!(out.commit, "_");
    }

    #[test]
    fn option_space_full_width_and_hz() {
        let mut e = engine();
        press(&mut e, KeyV);
        let out = e.key(
            Space,
            Mods {
                alt: true,
                ..Mods::NONE
            },
        );
        assert_eq!(out.commit, "ㅎ\u{3000}");
        // ⇧1 is the digit; 半角/全角 makes it full-width and keeps it staged
        shift(&mut e, Digit1);
        let out = press(&mut e, Backquote);
        assert_eq!(out.preedit, "１");
        assert!(out.consumed);
    }
}
