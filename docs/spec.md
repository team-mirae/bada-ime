# bada ime: spec

Status: **draft** (2026-11-07). Items marked **[decide]** are not settled: don't implement a guess
silently. The Bada layout itself (id 13) is designed in [bada-layout.md](bada-layout.md).

## 1. Names

| | Project | Full | Short |
|---|---|---|---|
| id | `bada-ime` | | |
| Korean | | 바다 입력기 | 바다 |
| Japanese | | バダIME | バダ (always katakana) |
| English | | bada ime | bada (always lowercase) |

Repo: `team-mirae/bada-ime`, open source, MIT, copyright Mirae Research.

Most of what bada ime needs already exists in **JIS Moonshot** (`team-mirae/jis-moonshot`) and
**JIS Korean** (`team-mirae/jis-korean`): the engine, the Hangul automaton, the kana layout and
automaton, the predictor and its corpora tools, and the macOS InputMethodKit shell. bada ime
reorganises them around two modes (muhenkan, henkan) and a pool of layouts.

## 2. Scope

- **macOS only.** No Windows, no web playground as a product (a design playground is fine).
- **JIS keyboards only** (日本語配列). Keys are physical positions with JIS legends; on any other
  keyboard layout the symbol keys type the wrong symbols. No ANSI/ISO modes.
- **Caps Lock is out of scope.** bada ime never handles it. If the user has set macOS to use Caps
  Lock for switching input sources, macOS turns it into 英数 / かな itself.

## 3. Vocabulary

Canonically **무변환 모드** / **변환 모드**; in the project, **muhenkan mode** and **henkan mode**
(Japanese readings).

| Term | Meaning |
|---|---|
| layout | A mapping from keystrokes to text, with an integer **layout id** (§5) |
| muhenkan layout | A layout that types a letter or sends a keycode per key. It **cannot compose**: no preedit |
| henkan layout | A layout that composes, as CJK input methods do: keys build a preedit, which is committed |
| deterministic | A henkan layout whose keys have exactly one reading (every Korean layout here) |
| non-deterministic | A henkan layout with several readings for the same keys (Japanese: kana → kanji) |
| candidate | A layout enabled in henkan mode. Muhenkan layouts can be candidates too |
| interpretation | One reading of the staged keys, by one candidate (or one of its conversions) |

## 4. Modes and input sources

### 4.1 Two input sources, no more

bada ime registers exactly two macOS input modes, and never more:

| Input source | Reports as | Role |
|---|---|---|
| **Bada Muhenkan** | English (`en`, Roman) | muhenkan mode |
| **Bada Henkan** | Japanese (`ja`) | henkan mode |

macOS routes 英数 to the current English/Roman source and かな to the Japanese one. Layout
switching inside a mode is bada ime's own state, not a macOS input source.

**Setup after install** (from JIS Moonshot's `mac/README.md`): add **ABC – Extended**, then remove
**ABC**, and remove every other Japanese input method. Then 英数 always lands in Bada Muhenkan and
かな in Bada Henkan. (ABC – Extended stays installed because macOS keeps one plain keyboard; it
still types in password fields.)

### 4.2 Muhenkan mode

QWERTY passthrough: practically the same as JIS English in JIS Moonshot. Only muhenkan layouts run
here; the active one is the **current muhenkan layout** (today only id 0 exists). Nothing composes,
nothing is staged.

### 4.3 Henkan mode

- **No sub-modes.** Every enabled candidate reads every keystroke at the same time, and the
  interpretations are **ranked as you type**; the top one shows in the preedit.
- With one candidate enabled there is no ambiguity, unless that candidate is non-deterministic
  (Japanese).
- Muhenkan layouts can be candidates. Enabling **0, 12, 20** reproduces today's JIS Moonshot
  (QWERTY + Moonshot Korean + JIS kana, predicted).
- The ranker is JIS Moonshot's predictor, generalised from its fixed three readings to the set of
  enabled candidates. **[decide]** how a candidate the predictor was never trained on (e.g. Bada,
  Romaji) is scored until it has a model.

## 5. Layouts

### 5.1 Id ranges

| Range | Use |
|---|---|
| 0–9 | Latin |
| 10–19 | Korean |
| 20–29 | Japanese |
| 30–49 | Sino |
| 50–99 | other languages |
| 100–149 | symbols, emoji, braille, IPA, other |
| 150–199 | reserved by Mirae Research |
| 200–`UINT32_MAX` | free to use |

Ids are `u32`. **Development / debug** ranges (re-numbered when merged to main): 300–399 Korean,
400–499 Japanese, 500–599 Sino, 600–999 other.

### 5.2 Catalog

| Id | Name | Display name | Kind | From |
|---|---|---|---|---|
| 0 | JIS QWERTY | `QWERTY` | muhenkan | JIS Korean / JIS Moonshot `A_QWERTY` (JIS English) |
| 10 | JIS KTS | `Korean (2-set)` | henkan, deterministic | standard KS X 5002 2-set on the JIS letter keys, JIS symbols untouched |
| 11 | JIS Korean Legacy | `Korean (legacy-korean)` | henkan, deterministic | JIS Korean Mode A |
| 12 | JIS Moonshot | `Korean (legacy-moonshot)` | henkan, deterministic | JIS Moonshot Mode A's Korean reading (moonshot.md §9) |
| 13 | JIS Bada | `Korean (bada)` | henkan, **ranked** (token keys, n-grams) | new: [bada-layout.md](bada-layout.md) |
| 18 | JIS Korean Roman | `Korean (Roman)` | henkan, ranked (the default reading shows; other readings offered) | new: [korean-roman.md](korean-roman.md) |
| 19 | JIS Kana Korean | `Korean (Kana)` | henkan, ranked (the table's reading shows; other pronunciations offered) | new, minor: [kana-korean.md](kana-korean.md) |
| 20 | JIS Kana | `Japanese (Kana)` | henkan, non-deterministic (kanji conversion) | JIS Moonshot `A_KANA`: [japanese.md](japanese.md) |
| 21 | JIS Romaji | `Japanese (Romaji)` | henkan, non-deterministic (kanji conversion) | new: [japanese.md](japanese.md) |

Every layout declares whether it is muhenkan or henkan.

### 5.3 Settings

- the **enabled set** of henkan candidates;
- the **default muhenkan layout** (英数 enters it);
- the **default henkan target** for HYPER+かな (§6.1), and the **cycle order** for MEH+かな.
  Without a user order: integer id ascending.

## 6. Keys

**HYPER** = ⌘⌃⌥⇧. **MEH** = ⌃⌥⇧.

### 6.1 Everywhere

| Key | Action |
|---|---|
| 英数 | Enter muhenkan mode, on the **default** muhenkan layout. From henkan mode, commit what shows first |
| かな | Enter henkan mode. Already in henkan mode: see §6.3 |
| HYPER+かな | **In henkan:** back to the most recently used muhenkan layout. **In muhenkan:** back to the most recently used henkan; else the user's default; else the lowest enabled id. A muhenkan layout chosen as a candidate inside henkan mode **does not count** as use of that muhenkan layout |
| MEH+かな | Cycle to the next enabled input source, in the user's order, else id ascending |

**[decide]** "most recently used henkan" and MEH+かな's "input source" imply more than one henkan
target, while §4.3 says henkan mode has no sub-modes. Which of these is meant?

- (a) a **henkan target** is either "all enabled candidates" or one candidate locked alone, and
  MEH+かな cycles through muhenkan layouts and those targets;
- (b) there are several named **henkan profiles** (each an enabled set), and the hotkeys pick a
  profile;
- (c) only one henkan exists, so HYPER+かな from muhenkan always just enters henkan, and MEH+かな
  cycles muhenkan layouts only.

### 6.2 In muhenkan mode

| Key | Action |
|---|---|
| ⌥Space | `_` |
| ⌥ + letter | its **variants**, as JIS English did (jis-korean spec §3.6.3): the letter shows with a row of accented variants (⌥e: è é ê ë ē ė ę; ⇧ for capitals), none chosen. 1–9, 0 pick; the same ⌥ + letter again or the arrows choose; Enter commits; Esc / ⌫ cancel; any other key keeps what shows and goes on. A letter without variants types macOS's ⌥ character. In terminals ⌥ goes to the app untouched |
| 半角/全角 | **full-width** applied to the selection |
| ⇧半角/全角 | **half-width** applied to the selection |

半角/全角 is the `Backquote` position (left of 1 on a full JIS board, at the bottom left on the HHKB
JIS; Apple JIS boards don't have it). The conversion rules are JIS Moonshot's (moonshot.md §4.1). **[decide]** with no selection:
JIS Moonshot falls back to the word under the caret; keep that, or do nothing?

### 6.3 In henkan mode

| Key | Action |
|---|---|
| ⇧Enter | Swallow Shift and Enter, **commit the preedit verbatim**, and do nothing else: never a newline, even where ⇧Enter normally makes one |
| Space, Enter, Esc, Tab, … | Commit the preedit as shown, **then** perform the key's normal action |
| かな, Tab, ↓ | Next entry in the suggestion menu (§6.4) |
| ⇧かな, ⇧Tab, ↑ | Previous entry; from entry 0 into the negative entries (§6.4) |
| F keys | Per the F-key bindings (§6.5); by default swallowed, doing nothing |
| ⌥Space | `　` (U+3000, full-width space) |
| 半角/全角 | **full-width**, applied to the unstaged text (the preedit) or else the selection |
| ⇧半角/全角 | **half-width**, the same way |

Rules that follow from the table:

- With a preedit, Tab / ⇧Tab cycle; they don't commit-then-tab. The "commit, then act" Tab only
  applies when **[decide]**: never (Tab always cycles when something is staged), or after the
  first cycle step? Proposed: Tab with a preedit always cycles; with none, it goes to the app.
- With **nothing staged**, Space, Enter, Esc, Tab, arrows and ⇧Enter go to the app unchanged
  (proposed; **[decide]** for ⇧Enter, which the table says is always swallowed). かな with nothing
  staged does nothing.
- Esc **commits** (it doesn't cancel), as the table says. Recorded because most IMEs cancel on
  Esc.
- Cmd / Ctrl shortcuts: commit first, then pass through (as JIS Korean §3.5).

### 6.4 The suggestion menu (negative indexes)

The interpretations of the staged keys form one menu with **signed indexes**:

| Index | Entry |
|---|---|
| … −2, −1 | **Script forms**: the same reading written another way, defined by the method |
| **0** | The top-ranked interpretation: what shows while typing |
| 1, 2, … | The other ranked interpretations: kanji conversions and mixed suggestions, every enabled candidate's readings together |

- Typing always lands on **0**. Next (かな, Tab, ↓) goes up from there; previous (⇧かな, ⇧Tab, ↑)
  goes down, and **below 0 into the negative entries**.
- Japanese methods (20 Kana, 21 Romaji) define **−1 hiragana** and **−2 katakana**: from a fresh
  reading, ⇧Tab once is hiragana, twice katakana.
- This **replaces F6 / F7** as the way to get hiragana and katakana. JIS Moonshot's F6–F10 (hiragana,
  katakana, kanji, full-width, half-width) are **not** ported as fixed keys; the F keys are reserved
  for §6.5.
- The menu shows the negative entries above entry 0 (a richer list than macOS's candidate window:
  the shell draws it).
- **One method, at most two entries** (owner, 2026-11-07). Only one enabled method contributes
  negative entries, and it contributes at most two (−1, −2). The slot exists for Japanese script
  override, but any method may offer it; the **user selects which method** in settings. Proposed
  default: the lowest-id enabled method that offers them (20 Kana before 21 Romaji).
- Other conversions are **not** negative entries: half-width comes from ⇧半角/全角, the old F8–F10
  conversions from F-key actions (§6.5), and the plain English reading is a ranked entry (≥ 0)
  whenever QWERTY (0) is enabled.
- **No wrap.** Next on the last entry and previous on the lowest negative entry do nothing.
- Committing commits the entry shown, whatever its index.

### 6.5 F-key actions

**F6–F10 are reserved by bada ime** (macOS's Japanese input methods already use them for kana
conversion, so apps don't rely on them while a Japanese input source is on).

- **Methods declare actions.** Every henkan method (layout) may publish a list of **F-key actions**:
  named operations on the staged text or the menu. Publishing them is **optional by design**: a
  method with none is valid, and the core never assumes any. The Japanese methods publish the
  usual macOS / MS-IME set:

  | Action (Japanese methods) | Usual key |
  |---|---|
  | `hiragana` | F6 |
  | `katakana` | F7 |
  | `halfwidth-katakana` | F8 |
  | `fullwidth-latin` | F9 |
  | `halfwidth-latin` | F10 |

  The "usual key" is only a hint for a settings preset; nothing is bound by default.
- **The user binds keys to actions** in settings: each intercepted F key is one of
  - **swallowed** (the default): the key is eaten and does nothing;
  - **passthrough**: the key goes to the app unchanged;
  - **`<method>.<action>`**: that method's action. If the method isn't enabled, or has nothing to
    act on, the key is swallowed.
- **Intercept F keys** (setting, on by default): off, every F key passes through to the app, and no
  binding applies.
- A settings preset "Japanese IME (F6–F10)" binds the five actions above to their usual keys.
- **Only F6–F10** are intercepted: a carry-over from Japanese input methods. Every other F key
  always goes to the app.
- **Modifiers are up to the method.** The user binds plain F6–F10; what ⇧ / ⌥ / ⌃ + that F key does
  is decided by the method its plain key is bound to (a method may define modified actions, or
  none). Unbound or undefined: swallowed, like the plain key.
- **Muhenkan mode passes F keys through**, always (no preedit, no methods).
- With nothing staged, an action applies to nothing and the key is swallowed (a method may define
  otherwise, e.g. act on the selection).

## 7. Build and layout of the repo

| Path | What |
|---|---|
| `/engine` | Platform-independent core (Rust, `bada-engine`): layouts, composition, the menu, F-key actions. Event in → output out, no I/O. C ABI in `include/bada.h` (JSON in and out) |
| `/engine/data` | Shipped data: Korean word counts (`ko-words.tsv`, built by `engine/tools/build_ko_words.py`), English words, hanja, OECD names, with their licenses |
| `/mac` | InputMethodKit shell (Swift, no Xcode): the two input modes (§4.1), the 英数 / かな key tap, the menu window, the settings window |
| `/docs` | This spec, the Bada layout design, saved layout candidates |
| `/tools/analysis`, `/tools/render`, `/tools/editor` | Corpus analysis and layout optimisation, the layout renderer, the layout editor |

All behaviour lives in the engine; the shell translates keys, switches input sources and applies
the output.

- Engine: `cargo test` in `/engine` (unit tests, plus `tests/real_data.rs` on the shipped data).
- macOS: `make` in `/mac` builds `build/BadaIME.app` (universal); `make install` copies it to
  `~/Library/Input Methods`, registers it (enabling both modes on a first install) and adds a login
  agent so the 英数 / かな tap runs from login. Then follow §4.1's setup.
- Bundle id `me.sjun.inputmethod.Bada`, modes `….Muhenkan` and `….Henkan`.

### 7.0 What v0.1 builds

- Layouts **0** (QWERTY), **10** (2-set), **11** (JIS Korean), **12** (JIS Moonshot), **13** (Bada, bada-layout.md §16), **18** (Korean Roman), **19** (Kana Korean),
  **20** (Kana) and **21** (Romaji). 20 and 21 convert to kanji (Mozc's dictionary, docs/japanese.md)
  and give the negative entries (−1 hiragana, −2 katakana); their menu rows carry romaji / kana notes.
- 11 and 12 are the Korean readings only, deterministic: the keys, implied ㅇ, the double press and
  ⇧ + vowel of JIS Korean §3–§4.2, and JIS Moonshot §9's slots and ⇧ + consonant. Not ported: their
  predictions, hanja, fixed F6–F10, and JIS Korean's ⇧Z as F10 (⇧Z is ㅋ here). Both publish two
  F-key actions (§6.5), bound to nothing by default: `commit` (this layout's reading as typed,
  whatever ranks first) and `commit-bare` (the same with the implied ㅇ dropped, as JIS Korean's F6:
  ㅏ → ㅏ, 가어 → 가ㅓ, 와 → ㅘ; a typed or asked-for ㅇ and a syllable with a final stay).
- Ranking is rules over word counts (a known word beats an unknown one; 초성 strings rank words by
  count), **not yet n-grams**.
- HYPER+かな and MEH+かな both switch to the other mode: with one muhenkan layout and one henkan
  there is nothing else to go back to or cycle through. The user cycle order (§5.3) isn't in the
  settings window yet.
- Default settings: henkan candidates = [13]; F6–F10 intercepted and swallowed, nothing bound.

### 7.1 Keyboards drawn by the renderer

Both have Control left of A. Bottom rows (1u each but the space):

- **HHKB JIS** (15u): fn · 0.25u gap · 半角/全角 · ⌥ · ⌘ · 英数 · space (2.5u) · かな · ⌘ · ⌥ · fn ·
  0.25u gap · ← ↓ →. ↑ sits in the Shift row, left of right Shift.
- **Apple-style JIS** (14.5u, MacBook): caps · ⌥ · ⌘ · 英数 · space (4.5u) · かな · ⌘ · 🌐 · ← ↓ →, with
  ↑ stacked over ↓ at half height; no right ⌥. The left edge is 0.5u shorter than a full board's:
  no 半角/全角 key (1 is 1.5u), tab 1u, control 1.25u, left shift 1.75u.

## 8. 초성 diagnostics and the user dictionary

초성 input is ambiguous enough that, in practice, every user who types it needs data of their own.
Two things provide it.

### 8.1 초성 diagnostics

A collector in the engine (`engine/src/diag.rs`) counts, **in henkan mode only**, for each Korean
layout that types initials on their own keys (10–13):

- **typed:** each committed syllable whose initial was typed: the initial's **key and Shift** (two
  keys that type the same jamo are two inputs: Bada's ㅅ on S and O, ㅇ on F and M, ㄷ on K and V,
  ㅆ on W and P, ㅃ on R and `;`), the vowel, the final, and the final's key;
- **choseong:** each 초성 prediction chosen: each typed initial's key, and the vowel and final of
  the syllable it became.

Counts only, never text. On by default (bada ime Settings → 초성 diagnostics; Clear empties it);
saved every 30 seconds to `~/Library/Application Support/bada-ime/choseong-diagnostics.json`
(`bada-choseong-diagnostics/1`: rows of the fields above with a `count`), and read back at launch.
`tools/analysis/choseong_report.py` summarises it. **[decide]** whether the counts feed the
ranking directly (a per-key syllable model for 초성 input) once there are enough.

### 8.2 The user dictionary

`~/Library/Application Support/bada-ime/dictionary.json`: positive words, negative words (soft or
strong) and 초성 shortcuts, for every layout. The contract is
[dictionary-contract.md](dictionary-contract.md), with a JSON Schema and an example; the
interactive setup is [interactive-prompts/](interactive-prompts/README.md). Loaded at launch and on
the next activation after the file changes; the input menu's "Open Dictionary Folder" opens its
folder.

## 9. Open questions

Collected from above:

1. §4.3 scoring of candidates without a trained model.
2. ~~§5.2 whether Bada is deterministic~~: ranked (owner, 2026-11-07).
3. §6.1 what "most recently used henkan" and MEH+かな's sources are: (a), (b) or (c).
4. §6.2 半角/全角 with no selection in muhenkan.
5. §6.3 Tab: cycle versus commit-then-tab; keys with nothing staged.
6. ~~§6.4 negative entries, menu ends~~: settled 2026-11-07.
7. ~~§6.5 F keys~~: settled 2026-11-07.
