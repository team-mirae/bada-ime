//! The engine: muhenkan and henkan modes (spec §4), the signed suggestion menu (§6.4), the keys
//! (§6), F-key actions (§6.5) and Bada's actions (bada-layout.md §16.1).
//!
//! Event in → output out. The shell owns input-source switching (英数, かな, HYPER/MEH+かな) and
//! the host text; the engine says what to commit, what to show and whether it took the key.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::data::Data;
use crate::japanese;
use crate::key::{Key, Mods};
use crate::layout::{self, BADA, KANA, KANA_KOREAN, KOREAN_ROMAN, KTS, LEGACY_KOREAN, LEGACY_MOONSHOT, Press, QWERTY, ROMAJI};
use crate::width::{Width, convert};

/// The user's settings (spec §5.3, §6.4, §6.5), as the shell stores them.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    /// Henkan candidates, in the user's order (earlier wins ties).
    pub enabled: Vec<u32>,
    pub default_muhenkan: u32,
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
            default_muhenkan: QWERTY,
            negative_method: None,
            intercept_f: true,
            fkeys: HashMap::new(),
        }
    }
}

/// The F-key actions each method publishes (§6.5): (method id, action id, label).
pub const ACTIONS: &[(u32, &str, &str)] = &[
    (
        BADA,
        "raw",
        "Raw jamo: no implied ㅇ or ㅏ, no 종성, no predictions",
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

#[derive(Clone, Debug)]
struct Entry {
    text: String,
    score: f64,
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
        }
    }

    pub fn set_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.settings.enabled.retain(|&id| layout::is_known(id));
        self.recompute();
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Switching mode commits what's staged (the shell inserts the returned text).
    pub fn set_mode(&mut self, mode: Mode) -> Output {
        let mut out = Output::default();
        if mode != self.mode {
            out.commit = self.take_shown();
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
        self.clear();
        text
    }

    // ---------- readings ----------

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
            self.entries.push(Entry { text, score: 0.0 });
            return;
        }
        let enabled = self.settings.enabled.clone();
        let mut entries: Vec<Entry> = Vec::new();
        for (rank, &id) in enabled.iter().enumerate() {
            // Earlier in the user's order wins ties.
            let tie = -(rank as f64) * 0.001;
            if !self.presses.iter().all(|&p| layout::takes(id, p)) {
                continue;
            }
            match id {
                QWERTY => {
                    let text = layout::qwerty(&self.presses);
                    let letters = text.chars().all(|c| c.is_ascii_alphabetic());
                    let score = if self.data.is_english(&text) {
                        90.0
                    } else if letters {
                        30.0
                    } else {
                        20.0
                    };
                    entries.push(Entry {
                        text,
                        score: score + tie,
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
                    let count = self.data.korean_count(&default);
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
                    });
                    let mut known: Vec<(String, u32)> = all[1..]
                        .iter()
                        .map(|r| (r.clone(), self.data.korean_count(r)))
                        .filter(|r| r.1 > 0)
                        .collect();
                    known.sort_by(|a, b| b.1.cmp(&a.1));
                    for (i, (text, _)) in known.into_iter().take(8).enumerate() {
                        entries.push(Entry {
                            text,
                            score: score + tie - 0.01 * (i + 1) as f64,
                        });
                    }
                }
                KTS | LEGACY_KOREAN | LEGACY_MOONSHOT | BADA => {
                    if let Some(tokens) = layout::choseong_tokens(id, &self.presses) {
                        let literal = layout::tokens_text(&tokens);
                        let count = self.data.korean_count(&literal);
                        let score = if tokens.len() == 1 {
                            200.0
                        } else if count > 0 {
                            101.0 + (count as f64).ln()
                        } else {
                            50.0
                        };
                        entries.push(Entry {
                            text: literal,
                            score: score + tie,
                        });
                        if tokens.len() >= 2 {
                            for (word, count) in self.data.choseong_words(&tokens, 9) {
                                entries.push(Entry {
                                    text: word,
                                    score: 95.0 + (count as f64).ln() + tie,
                                });
                            }
                        }
                    } else {
                        let text = layout::compose_presses(id, &self.presses);
                        let count = self.data.korean_count(&text);
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
                        });
                    }
                }
                KANA | ROMAJI => {
                    let text = kana_reading(id, &self.presses);
                    let loose = text.contains(japanese::DAKUTEN)
                        || text.contains(japanese::HANDAKUTEN)
                        || text.chars().any(|c| c.is_ascii_alphabetic());
                    let score = if self.data.is_japanese(&text) {
                        92.0
                    } else if loose {
                        30.0
                    } else {
                        70.0
                    };
                    entries.push(Entry {
                        text,
                        score: score + tie,
                    });
                }
                _ => {}
            }
        }
        // Script forms (§6.4): one method, at most two entries: −1 hiragana, −2 katakana.
        let method = self
            .settings
            .negative_method
            .filter(|m| enabled.contains(m) && SCRIPT_FORM_METHODS.contains(m))
            .or_else(|| {
                enabled
                    .iter()
                    .copied()
                    .filter(|m| SCRIPT_FORM_METHODS.contains(m))
                    .min()
            });
        if let Some(method) = method.filter(|&m| self.presses.iter().all(|&p| layout::takes(m, p)))
        {
            let hira = kana_reading(method, &self.presses);
            self.negatives = vec![
                Entry {
                    text: hira.clone(),
                    score: 0.0,
                },
                Entry {
                    text: japanese::katakana(&hira),
                    score: 0.0,
                },
            ];
        }
        // One row per text, at its best score.
        entries.sort_by(|a, b| b.score.total_cmp(&a.score));
        let mut seen = std::collections::HashSet::new();
        entries.retain(|e| seen.insert(e.text.clone()));
        self.entries = entries;
    }

    fn finish(&mut self, mut out: Output) -> Output {
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
            out.menu = self
                .negatives
                .iter()
                .rev()
                .chain(self.entries.iter())
                .map(|e| e.text.clone())
                .collect();
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
        if mods.is_shortcut() {
            return out;
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
            let any = self.over.is_some()
                || self
                    .settings
                    .enabled
                    .iter()
                    .any(|&id| layout::takes(id, press));
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
        if !self.settings.intercept_f {
            return pass;
        }
        let binding = self
            .settings
            .fkeys
            .get(&format!("F{f}"))
            .cloned()
            .unwrap_or_else(|| "swallow".into());
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
        if !self.settings.enabled.contains(&method)
            || !ACTIONS.iter().any(|a| a.0 == method && a.1 == action)
        {
            return swallow;
        }
        self.action(method, action)
    }

    /// A method's action (bada-layout.md §16.1; Japanese: docs/japanese.md).
    pub fn action(&mut self, method: u32, action: &str) -> Output {
        let mut out = Output {
            consumed: true,
            ..Output::default()
        };
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

    fn engine() -> Engine {
        let data = Data::parse(
            "안녕하세요\t50\n안녕\t40\n한국\t60\n항구\t5\n아니하세요\t3\n한\t30\nㅋㅋ\t90\n감사합니다\t70\n학교\t20\n",
            "hello\nhan\n",
            "학교\t學校\t\n학\t學\t배울 학\n",
            "美\t미국\n英\t영국\n",
        );
        Engine::new(Arc::new(data))
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
        press(&mut e, KeyL);
        press(&mut e, Comma);
        let out = press(&mut e, KeyH);
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
        press(&mut e, KeyL);
        press(&mut e, Comma);
        let out = e.key(Enter, Mods::SHIFT);
        assert_eq!(out.commit, "하");
        assert!(out.consumed);
        // nothing staged: the app's
        assert!(!e.key(Enter, Mods::SHIFT).consumed);
    }

    #[test]
    fn choseong_predicts_and_shift_narrows() {
        let mut e = engine();
        for k in [KeyF, KeyH, KeyL, KeyS, KeyF] {
            press(&mut e, k);
        }
        let out = press(&mut e, Tab);
        assert!(out.menu.contains(&"안녕하세요".to_string()));
        assert_eq!(out.menu[0], "안녕하세요");
        let mut e = engine();
        // one consonant shows itself
        assert_eq!(press(&mut e, KeyF).preedit, "ㅇ");
        // ㅇㄴ⇧ㅎㅅㅇ: ⇧ㅎ = 하, still 초성 input
        let mut e = engine();
        press(&mut e, KeyF);
        press(&mut e, KeyH);
        shift(&mut e, KeyL);
        press(&mut e, KeyS);
        let out = press(&mut e, KeyF);
        assert_eq!(out.preedit, "안녕하세요");
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
        for k in [KeyF, KeyH, KeyL, KeyS, KeyF] {
            press(&mut e, k);
        }
        let n = e.entries.len();
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
        press(&mut e, KeyL);
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
        for k in [KeyL, Comma, KeyH] {
            press(&mut e2, k);
        }
        assert_eq!(press(&mut e2, F8).preedit, "ㅎㅏㄴ");
        let mut e3 = engine();
        e3.set_settings(e.settings().clone());
        for k in [KeyL, Comma, KeyH] {
            press(&mut e3, k);
        }
        assert_eq!(press(&mut e3, F9).preedit, "l,h");
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
        for k in [KeyF, KeyH, KeyL, KeyS, KeyF] {
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
        for k in [KeyL, Comma, KeyJ, KeyJ, Digit9] {
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
        press(&mut e, KeyL);
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
