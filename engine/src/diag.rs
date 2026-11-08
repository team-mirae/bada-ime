//! 초성 diagnostics (spec §8): which vowel and final the user attaches to each initial, per key.
//!
//! Counts only, never text. One collector per process, shared by every engine; off until the shell
//! turns it on (a setting, on by default); the shell loads it at launch and saves it now and then. Recorded on commits in henkan mode, from the Korean
//! layouts that type initials on their own keys (10–13):
//!
//! - **typed**: a syllable composed from its keys: the initial's key (and Shift), the vowel, the
//!   final and the final's key. Two keys that type the same jamo (Bada's ㅆ on 2 and on `:`) are two
//!   inputs.
//! - **choseong**: a 초성 prediction chosen: each typed initial's key, and the vowel and final of
//!   the syllable it became.

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::layout::{Detail, Press};

/// One observation's identity; its count is the map's value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Observation {
    pub layout: u32,
    /// "typed" or "choseong".
    pub source: String,
    pub key: String,
    pub shift: bool,
    pub initial: char,
    pub vowel: char,
    /// The final jamo, or "" for none.
    pub final_: String,
    /// The final's key ("" when there's none, or for a 초성 prediction: the user typed none).
    pub final_key: String,
    pub final_shift: bool,
}

#[derive(Default)]
struct Collector {
    enabled: bool,
    counts: BTreeMap<Observation, u64>,
    /// Changes since the last export.
    changes: u64,
}

static COLLECTOR: Mutex<Option<Collector>> = Mutex::new(None);

fn with<T>(f: impl FnOnce(&mut Collector) -> T) -> T {
    let mut guard = COLLECTOR.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Collector::default))
}

pub fn set_enabled(on: bool) {
    with(|c| c.enabled = on);
}

fn add(o: Observation) {
    with(|c| {
        if c.enabled {
            *c.counts.entry(o).or_insert(0) += 1;
            c.changes += 1;
        }
    });
}

/// A commit of composed syllables.
pub fn typed(layout: u32, presses: &[Press], details: &[Detail]) {
    for d in details {
        let (Some(ini), fin) = (presses.get(d.ini_press), d.fin_press.and_then(|i| presses.get(i))) else { continue };
        add(Observation {
            layout,
            source: "typed".into(),
            key: ini.key.code().into(),
            shift: ini.shift,
            initial: d.ini,
            vowel: d.vowel,
            final_: d.fin.map(String::from).unwrap_or_default(),
            final_key: fin.map(|p| p.key.code().to_string()).unwrap_or_default(),
            final_shift: fin.is_some_and(|p| p.shift),
        });
    }
}

/// A 초성 prediction chosen: press `i` became syllable `i` of `word`.
pub fn choseong(layout: u32, presses: &[Press], word: &str) {
    for (p, c) in presses.iter().zip(word.chars()) {
        let Some((ini, vowel, fin)) = crate::hangul::decompose(c) else { continue };
        add(Observation {
            layout,
            source: "choseong".into(),
            key: p.key.code().into(),
            shift: p.shift,
            initial: ini,
            vowel,
            final_: fin.map(String::from).unwrap_or_default(),
            final_key: String::new(),
            final_shift: false,
        });
    }
}

#[derive(Serialize, Deserialize)]
struct Row {
    #[serde(flatten)]
    observation: Observation,
    count: u64,
}

#[derive(Serialize, Deserialize)]
struct File {
    format: String,
    rows: Vec<Row>,
}

/// Everything collected, as JSON (`bada-choseong-diagnostics/1`).
pub fn export() -> String {
    with(|c| {
        c.changes = 0;
        let rows = c.counts.iter().map(|(o, &n)| Row { observation: o.clone(), count: n }).collect();
        serde_json::to_string(&File { format: "bada-choseong-diagnostics/1".into(), rows }).unwrap_or_default()
    })
}

/// Changes since the last export: the shell saves when there are any.
pub fn changes() -> u64 {
    with(|c| c.changes)
}

/// Loads saved counts (added to what's in memory). Malformed JSON is ignored.
pub fn import(json: &str) {
    let Ok(file) = serde_json::from_str::<File>(json) else { return };
    with(|c| {
        for row in file.rows {
            *c.counts.entry(row.observation).or_insert(0) += row.count;
        }
    });
}

pub fn clear() {
    with(|c| {
        c.counts.clear();
        c.changes += 1;
    });
}

