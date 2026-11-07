# JIS Bada (layout 13): design

Status: **design, round 2** (2026-11-07). Nothing here is settled; decisions are marked **[decide]**.

**Round 2 supersedes the key budget of round 1** (owner, 2026-11-07): Bada uses the **full JIS kana
space**, all 48 keys JIS Kana uses (1–0 − ^ ¥, Q–P @ [, A–L ; : ], Z–M , . / ろ). Shift on the 22
number and symbol keys is reserved for digits and symbols, as JIS Kana reserves it for its own
punctuation. Rounds 1's 26-key analysis (§1–§5) stays below for the record; §8 is current.

## 1. Goals and constraints

1. **Typing efficiency and less hand fatigue** drive every choice. KTS / 2-set compatibility is
   not a goal.
2. ~~Only the 26 letter keys~~ (round 1). Round 2: the 48 kana-space keys, Shift on the 22
   non-letter keys reserved for digits and symbols (§8).
3. **Efficient typing avoids Shift.** Shift is for rare things only.
4. Deterministic **[decide]**: one key sequence, one text. Henkan mode can rank several
   interpretations, but a layout that needs ranking to type plain Korean is harder to trust and to
   learn. Proposed: deterministic.

## 2. What the text costs (measured)

`tools/analysis/` reads 12M Hangul syllables, 3M each from news (Leipzig), formal writing
(petitions), reviews (NSMC) and conversation (korean-safe-conversation), from JIS Moonshot's
corpora. Shares are per syllable.

| | |
|---|---|
| initial ㅇ | **23.4%** of syllables: the most frequent initial by far (ㄱ is next, 12.1%) |
| …at word start / after an open syllable / after a final | 7.0% / 6.4% / 10.2% |
| a final consonant | 46.0% (ㄴ 14.8, ㄹ 8.1, ㅇ 7.6, ㄱ 4.5, ㅁ 3.0, ㅂ 2.4, ㅆ 2.1, ㅅ 1.2) |
| vowels | ㅏ 21.4, ㅣ 16.2, ㅡ 12.4, ㅓ 9.9, ㅗ 9.5, ㅜ 6.5, ㅕ 4.9, ㅐ 4.9, ㅔ 4.5, ㅘ 2.3, ㅛ 1.7, ㅢ 1.5, ㅚ 1.2, rest < 1 |
| 2-set | 2.53 keys per syllable; Shift on 2.0% of keys, half of it ㅆ (then ㄸ, ㄲ, ㅖ) |

Standalone jamo (ㅋㅋ, ㅎㅎ, ㅠㅠ) aren't counted yet; they matter in chat.

Takeaways:

- The **initial ㅇ** is the one big keystroke to win. It carries no sound; typing it is pure
  overhead.
- 2-set already puts Shift on only 2% of keys; the Shift that matters is **ㅆ** (했, 있, 었, 겠).
- ㅑ ㅠ ㅖ ㅒ are rare (under 1% each) and don't deserve keys of their own. ㅕ ㅐ ㅔ do.

## 3. Proposed rules

### 3.1 Implied ㅇ

A vowel typed **at the start of a word, or right after a syllable without a final**, starts a new
syllable on ㅇ: `ㅏ` → 아, `ㄱ ㅏ ㅏ` → 가아, `ㅜ ㄹ ㅣ` → 우리.

- If the vowel **combines** with the vowel before it, it combines (`ㅂ ㅗ ㅏ` → 봐), and the two
  syllables need an explicit ㅇ (`ㅂ ㅗ ㅇ ㅏ` → 보아). That costs 0.5% of syllables (보이, 주어,
  수이…), against 13.4% of syllables saved.
- After a final the vowel takes the final, as in 2-set (`ㅁ ㅓ ㄱ ㅓ` → 머거); 먹어 is still
  `ㅁ ㅓ ㄱ ㅇ ㅓ`. Moving this case (10.2% of syllables) would need a syllable-boundary key or
  separate final keys, and 26 keys have no room for either.
- The ㅇ key still works everywhere; nobody has to use the implied ㅇ.

Measured: 2.53 → **2.39 keys per syllable (−5.3%)**.

### 3.2 Vowels: keys and sequences

Keyed: ㅏ ㅓ ㅗ ㅜ ㅡ ㅣ ㅐ ㅔ ㅕ, plus two of ㅛ ㅘ ㅢ (§4). The rest are typed as sequences the
automaton joins, the way 2-set already joins ㅗ+ㅏ:

| Vowel | Typed |
|---|---|
| ㅘ ㅙ ㅚ | ㅗ ㅏ, ㅗ ㅐ, ㅗ ㅣ |
| ㅝ ㅞ ㅟ | ㅜ ㅓ, ㅜ ㅔ, ㅜ ㅣ |
| ㅢ | ㅡ ㅣ |
| **ㅑ ㅠ** | **ㅣ ㅏ, ㅣ ㅜ** (new) |
| **ㅖ ㅒ** | **ㅕ ㅣ, ㅑ ㅣ** (new; off Shift) |
| ㅛ | ㅣ ㅗ (new), if ㅛ has no key |

ㅣ-first pairs never occur inside a Korean syllable, so they can't be misread; with the implied
ㅇ, 이아 / 이우 take an explicit ㅇ like 보아.

### 3.3 Consonants

- 14 consonants keyed, with **ㅆ on its own key** (no Shift: the most frequent Shift in 2-set).
- **ㄲ ㄸ ㅃ ㅉ on Shift** of their plain consonant (0.5–0.9% of keys in total). Doubling a
  consonant can't type them deterministically: `ㄱ ㅏ ㄱ ㄱ ㅏ` is both 각가 and 가까.
- Double finals (ㄶ ㄺ ㅄ …) as in 2-set: two consonants in a row.

## 4. Candidate systems (measured)

Every system below was placed on the 26 keys by simulated annealing (`optimize.py`) under one
effort model: per-key cost (home row and index/middle fingers cheapest, pinky and bottom corners
dearest), same-finger bigrams penalised hardest, then same-hand row jumps, Shift at 2 extra. Effort
is per syllable; lower is better. The optimizer is free to mix consonants and vowels across hands;
it chose the 2-set split (consonants left, vowels right) every time.

| System | Keys/syl | Effort/syl | Shift | Same finger | Left hand |
|---|---|---|---|---|---|
| 2-set as it is | 2.527 | 3.351 | 2.02% | 2.86% | 58.2% |
| 2-set, keys re-placed | 2.527 | 3.105 | 2.02% | 1.89% | 58.2% |
| 2-set + implied ㅇ | 2.394 | 3.043 | 2.14% | 2.59% | 55.9% |
| **A** ㅆ ㅛ ㅘ keyed | 2.397 | 3.004 | 0.96% | 2.47% | 55.8% |
| **B** ㅆ ㅛ ㄲ keyed | 2.421 | 3.006 | **0.55%** | 2.47% | 55.1% |
| **C** ㅆ ㅛ ㅢ keyed | 2.406 | **2.987** | 0.95% | 2.03% | 55.7% |
| **D** ㅆ ㅑ ㅛ ㅠ keyed, ㅔ = ㅓ ㅣ | 2.445 | 3.024 | 0.94% | 1.94% | 54.5% |

Against 2-set: about **11% less effort, 5% fewer keys, half the Shift or less**. A, B and C are
within 1% of each other, which is inside the model's error: choose between them on feel.

### 4.1 Draft board: C

```
 Q  W  E  R  T    Y  U  I  O  P
 ㅌ ㄷ ㄹ ㅈ ㅆ   ㅢ ㅓ ㅡ ㅜ ㅛ
  A  S  D  F  G    H  J  K  L
  ㅅ ㅇ ㄴ ㄱ ㅎ   ㅔ ㅏ ㅣ ㅗ
   Z  X  C  V  B    N  M
   ㅍ ㅋ ㅁ ㅂ ㅊ   ㅐ ㅕ
```

Shift: ⇧ㄷ ㄸ, ⇧ㅈ ㅉ, ⇧ㄱ ㄲ, ⇧ㅂ ㅃ (⇧ㅅ is ㅆ too). Everything else on Shift is free.

This board is a first optimizer output, not a design: the effort model is generic (no row
stagger, no JIS-specific reach, no hand-specific weights), and placement should be re-run once the
model is tuned and the open questions below are answered.

## 5. What 26 keys can't do

- **Left-hand load stays ~55%.** Consonants (initials and finals) outnumber vowels, and a final
  followed by the next initial is a same-hand pair. Fixing that needs finals on the vowel hand
  (3-set style), which needs about 7 more keys.
- **먹어-type syllables (10.2%) keep their ㅇ**, for the same reason.

Both are open to a bigger idea; within 26 deterministic keys, §3 is about the ceiling.

## 6. Open questions

1. **[decide]** Deterministic, as proposed?
2. **[decide]** The implied ㅇ (§3.1), with its "combining wins" rule?
3. **[decide]** The two spare vowel slots and the tense consonants: A (ㅛ ㅘ), B (ㅛ ㄲ, least
   Shift), C (ㅛ ㅢ, least effort), or another mix.
4. **[decide]** Keep the consonants-left / vowels-right split? (The optimizer prefers it.)
5. **[decide]** Shift's free slots: leave empty, or use for rare syllables/endings?
6. Which keyboard to tune the effort model on (Mac JIS, HHKB JIS, …): stagger and reach differ.
7. Standalone jamo in chat (ㅋㅋ, ㅠㅠ): measure and give them a cheap path.

## 8. Round 2: the 48 kana-space keys

### 8.1 Shift on the number and symbol keys **[decide]**

Proposed: **⇧ + a number or symbol key types its unshifted JIS character** (⇧1 → 1, ⇧- → -,
⇧/ → /, ⇧ろ → \\), and **⇧⌥** types its shifted JIS character (⇧⌥1 → !, ⇧⌥/ → ?). Shift on the
26 letter keys stays free for Korean. Open: whether ⇧ should give the *shifted* character on some
keys instead (? and ! are frequent in chat; on `/` and `1` ⇧ could give ? and !).

### 8.2 What 48 keys make possible

With 48 keys there is room for **separate initial and final keys** (3-set style), which round 1
couldn't fit:

- the syllable boundary is always known, so **every initial ㅇ can be implied**, after a final too
  (`ㅁ ㅓ ㄱ(final) ㅓ` → 먹어): 23.4% of syllables lose a key, not 13.4%;
- no jamo jumps between syllables (no 도깨비불): a syllable is final the moment the next initial
  or vowel arrives;
- ㄲ ㄸ ㅃ ㅆ ㅉ get keys of their own: no Shift for Korean at all, if wanted.

### 8.3 Measured

Same corpora and method as §4, now on 48 keys plus Shift on the letters (`optimize48.py`). The
number row and the outer right-pinky keys are dear in the effort model.

| System | Units | Keys/syl | Effort/syl | Shift | Same finger | Same hand | Left hand |
|---|---|---|---|---|---|---|---|
| 2-set as it is (§4, 26 keys) | 26 | 2.527 | 3.351 | 2.02% | 2.86% | 18.6% | 58.2% |
| **2-set-48**: every vowel and tense consonant keyed, implied ㅇ | 40 | 2.325 | **2.870** | 0.11% | 1.41% | 21.9% | 49.9% |
| **3-set A**: 13 vowels, 16 finals | 48 | 2.267 | 2.988 | 1.30% | 0.80% | 22.2% | 47.4% |
| **3-set B**: 18 vowels, all 27 finals | 64 | **2.218** | 2.970 | 1.60% | 0.58% | 18.1% | 48.8% |
| **3-set C**: 15 vowels, 18 finals | 52 | 2.239 | 2.973 | 1.39% | **0.55%** | **16.9%** | 46.9% |

Reading it:

- Every 48-key system beats 2-set by 11–14% in effort and 8–12% in keys, and balances the hands
  (≈ 48% left against 58%).
- **3-set types 2–5% fewer keys and halves same-finger pairs**, but needs 48–64 keys against
  2-set-48's 40, so more of them land on the number row and the pinkies, and the generic model
  scores its effort ~4% higher. Which wins depends on how heavily the model weighs reach against
  keystrokes and same-finger pairs: this needs the owner's hands, not more annealing.
- The annealed 3-set boards mix initials and finals on both hands. That's hard to learn; a real
  3-set design would group them (initials on one hand, vowels and finals on the other, as 3-set
  390 and 3-final do). Next step: re-run with that constraint.

### 8.4 Open questions (round 2)

1. **[decide]** 2-set-48 (fewer keys to learn, lowest modelled effort) or 3-set (fewest keys,
   implied ㅇ everywhere, no jamo jumps)?
2. **[decide]** §8.1, Shift on the number and symbol keys.
3. **[decide]** For 3-set: group initials and finals by hand?
4. Tune the effort model on the HHKB JIS (the owner's daily board): especially how dear the number
   row and the keys right of P / L / ろ really are.

The drafts can be seen in the renderer (`tools/render`, §9).

## 9. Reproducing

```sh
cd tools/analysis
python3 -m venv .venv && .venv/bin/pip install numpy
python3 jamo_stats.py            # §2 frequencies
python3 ieung.py                 # §3.1 where initial ㅇ appears
.venv/bin/python optimize.py     # §4 systems and boards (26 keys)
.venv/bin/python optimize48.py   # §8 systems and boards (48 keys) -> out_48.json
python3 ../render/build_layouts.py   # renderer data, drafts included
open ../render/index.html
```

`tools/render/index.html` draws any layout on the **HHKB JIS** (dimensions from the owner's HHKB
Keycap Swap artifact) or a **generic Apple-style JIS** board, with Shift legends, the JIS legends,
usage heat for drafts, and SVG export. Board geometry is in `keyboards.js`.

Set `BADA_CORPORA` if JIS Moonshot's corpora aren't at `~/Projects/jis-moonshot/corpora`.

## 10. Round 3 idea: token keys (2026-11-07, exploring)

The owner's idea: a key needn't be one jamo. It can be a **token** that stands for several (`F` =
ㅎ / ㅍ / ㅃ), and the henkan ranker picks the word, as kana-to-kanji conversion does.

Combined with 3-set roles (separate initial, vowel and final keys, every compound vowel and double
final a single key) this means **one key per jamo slot**: about 2.2 keys per syllable, never more,
and the syllable shape is always certain; only which member of a token is meant is guessed.

`tools/analysis/tokens.py` merges jamo greedily (within a role, the pair that adds the least
ambiguity), and `tokens_eval.py` re-scores the groupings on the top 100k words (85.5% of running
words). "Wrong" = a more frequent word has the same keys, by word frequency alone, no context:

| Keys | Wrong at top-1 | Not in top 3 |
|---|---|---|
| 62 (every jamo its own key) | 0 | 0 |
| 42 | 0.10% | 0.00% |
| 34 | 0.28% | 0.00% |
| **30** | **0.44%** | 0.00% |
| **26** | **0.88%** | 0.02% |
| 22 | 1.86% | 0.09% |

The 30-key grouping (greedy, not tuned for learnability):

- initials (13): ㄱ ㄴ ㄷ ㄹ ㅁ ㅂ ㅅ ㅇ ㅈ ㅎ, ㄸ/ㄲ/ㅍ, ㅉ/ㅊ/ㅋ, ㅌ/ㅃ/ㅆ
- vowels (10): ㅏ, ㅓ, ㅗ, ㅣ/ㅒ, ㅐ/ㅢ, ㅕ/ㅔ, ㅘ/ㅚ, ㅜ/ㅠ/ㅞ, ㅡ/ㅟ/ㅑ, ㅙ/ㅛ/ㅖ/ㅝ
- finals (7): ㄹ, ㄱ/ㄺ, ㄴ/ㄲ/ㄼ/ㄵ, ㅁ/ㄷ/ㅀ, ㅂ/ㅌ, ㅅ/ㅊ/ㅄ/ㅍ/ㅈ/ㄻ, ㅆ/ㅇ/ㅎ/ㄶ

With the implied ㅇ the initial ㅇ key goes too: 29 keys, inside the letter area with room to
spare, so **every symbol key could keep its JIS character** again.

Caveats, not yet measured:

- **In-sample and unigram.** The vocabulary that scores the groupings also chose them, and the
  real ranker has context (better) but meets names, slang and new words outside the top 100k
  (14.5% of running words; worse). Next: held-out text, eojeol context, and the OOV share.
- **Learnability.** Greedy groups are arbitrary (ㅌ/ㅃ/ㅆ). Groups constrained to be mnemonic
  (sound families ㄱ/ㅋ/ㄲ, shapes) will cost some ambiguity; how much is the next measurement.
- **Escape hatch.** A deterministic way to force a member (e.g. Shift+token = its second member,
  or cycling inside a syllable) for names and ㅋㅋ.
- **Feel.** The preedit can change as you type, as in Japanese conversion; at 0.5–0.9% wrong that
  is one correction every ~110–230 words.

## 11. Round 3 idea: chords (2026-11-07, exploring)

The owner: simultaneous presses aren't claimed by any global setting, so Bada can take chords.

Korean suits chords better than most scripts: a syllable is already a fixed set of roles
(initial, vowel, final). With role-separated keys (§10) **one chord can type one syllable**, as
Korean stenotype machines do. And because the roles use different keys, the order of the keys
inside a syllable carries no information: a chord and a fast roll type the same thing.

Questions to settle before designing:

- **Hardware rollover.** A chord of 3–4 keys needs NKRO or at least ghost-free 4-key combinations on
  the keys used. To check: HHKB Hybrid Type-S over USB and Bluetooth, and Apple JIS boards.
- **InputMethodKit key-up.** Release-based strokes (a stroke ends when every key is up) need key-up
  events in the input controller (`recognizedEvents` with key-up). To verify per app in the first
  macOS spike; without it, chords have to be timing-based on key-down only.
- **Stroke boundaries under rollover.** Fast sequential typists overlap keys across syllables.
  Proposed rule: parse in press order by role (initial → vowel → final starts a new syllable
  whenever the role goes back), so a chord, a roll and plain sequential typing agree.
- **What chords are for.** (a) only speed: any order inside a syllable; (b) more symbols from fewer
  keys: e.g. vowels as chords of 3–4 keys, so fewer keys and fewer tokens; (c) whole-word or ending
  briefs (했습니다, 있는) as steno does.

## 12. Decisions (2026-11-07, the owner)

- **Ranked is fine.** Bada may be non-deterministic; n-grams trained on the corpora compensate for
  token ambiguity (§10). This supersedes "deterministic" in §1.
- **Home keys cost nothing.** A S D F and J K L ; carry **zero** effort in the model: the fingers
  rest there. (The model so far charged them 0.7–1.3.)
- **One token, several inputs.** A token may be reachable by more than one key or chord, and should
  be where it helps ergonomics (e.g. a frequent token on a home key and in a chord).
- **Chord limits** (after the NKRO check): normal typing chords **at most 2 keys**; very fast
  typing **at most 3**. Larger chords are out.
- **Chords are only an accelerator.** Every letter must also be typeable **one key at a time**
  (1KRO): **a chord is never required**. Navigating predictions with the arrow keys counts as
  1KRO too, so picking a non-top reading needs no chord either. So the sequential layer
  is complete on its own (§10's token keys), and chords add speed on top (§11 (a)–(c), within the
  2/3-key limits).

## 13. Token keys placed (effort model v2, 2026-11-07)

Model v2: A S D F J K L ; cost **zero**; Shift unused (1KRO); initial ㅇ always implied (a vowel
key after a vowel or final always starts a syllable, since every vowel token is one key).
`place_tokens.py` places §10's greedy groupings on the 48 keys. Standard 2-set re-scored under v2
for comparison.

| | Keys | Keys/syl | Effort/syl | Wrong top-1 (unigram) | Same finger | Pinky |
|---|---|---|---|---|---|---|
| 2-set as it is | 26 + Shift | 2.527 | 2.213 | 0 | 2.86% | 8.3% |
| tokens, step 32 | 29 | **2.215** | 1.869 (−16%) | 0.44% | 0.48% | 18.9% |
| tokens, step 36 | **25** | **2.215** | **1.737 (−22%)** | 0.88% | 0.44% | 18.3% |
| tokens, step 40 | 21 | 2.215 | 1.610 (−27%) | 1.86% | 0.90% | 14.9% |

Seen in the renderer (layouts 312–314):

- The optimizer put **vowels on the left hand and consonants on the right**, initials toward the
  centre and finals on the outer right (L ; : , . /). The number row went unused at 25 keys.
- At 25 keys, Q T Z X C B are empty: room for second inputs of frequent tokens (§12) or chords.
- **Pinky load doubled** (8% → 18%): with A and ; free, the model loads them. **[decide]** keep
  resting keys at zero for every finger, or give the pinky home keys a small cost.
- The groups are still greedy and unlearnable (§10); the next round constrains them.

## 14. Candidates

Hand-built in the layout editor and saved under `docs/layouts/`. Scored with
`tools/analysis/evaluate_layout.py` (model v2; duplicates taken by the cheapest route through each
word; ambiguity not scored).

| Candidate | Keys/syl | Effort/syl | Shift | Same finger | Same hand | Pinky | Left |
|---|---|---|---|---|---|---|---|
| 2-set (KTS), reference | 2.527 | 2.213 | 2.02% | 2.86% | 18.6% | 8.3% | 58.2% |
| 1, initial ㅇ explicit (as first saved) | 2.457 | 2.528 | 3.49% | 2.57% | 15.3% | 19.3% | 56.5% |
| 1 `bada-candidate-1.json`, **implied ㅇ** (owner, 2026-11-07) | **2.317** | 2.570 | 3.70% | 2.96% | 19.5% | 20.5% | 54.1% |
| **2** `bada-candidate-2.json` "초성입력가능" (owner, 2026-11-07; **preferred**) | 2.323 | 3.414 | 0 | 2.44% | 46.6% | 25.3% | 37.9% |

Candidate 1: 2-set-style consonants (초·종성 on one key) on the left, vowels on the right home row
with ㅕ ㅑ ㅛ ㅠ on Shift, every compound vowel and tense consonant keyed (number row), double
finals on the bottom right, and right-hand duplicates of ㄱ ㄴ ㄹ ㅁ ㅇ and left-hand duplicates
of ㅏ ㅣ. Implied ㅇ (2026-11-07; first saved with it off): a vowel at word start or after an
open syllable starts a syllable on ㅇ. Every compound vowel has a key, so **[decide]** whether
ㅗ+ㅏ still joins into ㅘ (then 보아 needs an explicit ㅇ) or never joins (then `ㅂ ㅗ ㅏ` is 보아).
The implied ㅇ saves 5.7% of keys but scores slightly *worse* in effort: ㅇ sat on F, a resting key
the model charges nothing, and dropping it puts more vowel → vowel pairs on the right hand. It puts jamo on Shift of four symbol keys (0 ⇧ㅙ, ] ⇧ㄱ, / ⇧ㅀ,
ろ ⇧ㄿ), which §8.1 reserves for symbols.

Candidate 2 ("초성입력가능", the owner's preferred): every consonant on its own unshifted key in
both roles (2-set style), across the home and Q rows of both hands (S D F G | H J K L hold ㅅ ㅈ
ㅇ ㄹ | ㄴ ㄱ ㄷ ㅎ); vowels on the edges (A ㅗ, Z ㅜ, Q ㅕ, : ㅓ, , ㅏ, . ㅣ, / ㅡ, [ ㅔ, ] ㅐ) and
the number row (compound vowels, ㅑ ㅠ ㅛ, second ㅏ on 1 and 3); duplicates for ㅅ ㅆ ㅇ ㄷ ㅃ;
no Shift; implied ㅇ. Double finals other than ㅄ are typed as two consonants.

Its model scores come from the vowels: ㅏ (21% of syllables) sits on `,` 1 3, ㅣ (16%) on `.`, ㅡ
(12%) on `/` and ¥, ㅗ on A, ㅜ on Z, so the pinkies carry 25% and consonant → vowel pairs land on
the same hand (46.6% same-hand, against 2-set's 18.6%). 초성 input doesn't depend on that
placement (§15): any layout with every initial on an unshifted consonant key supports it.

## 15. 초성 input (owner, 2026-11-07)

Bada accepts **초성-only typing** as an abbreviation, ranked like any other reading:

- **Only consonants typed** in the segment: the keys are read as a 초성 string and the menu offers
  the words that start with those 초성 (ㅇㄴㅎㅅㅇ → 안녕하세요), ranked by the n-gram model. The
  literal jamo string is always an entry too (ㅋㅋㅋ, ㅎㅎ, ㅇㅇ, ㅅㄱ are common on their own).
- **At least one 중성** in the segment: normal composition, as in any Korean method; the 초성
  reading is dropped.
- **Requirement on the layout:** every initial consonant on an unshifted key, not shared with a
  vowel. Candidate 2 meets it; so do 2-set and candidate 1.

How much a 초성 string alone says (word frequency only, no context; the n-gram ranker does better):

| Word length | Share of words | Right at top-1 | In top 5 |
|---|---|---|---|
| 1 syllable | 9.3% | 39.7% | 75.1% |
| 2 | 25.9% | 31.6% | 60.0% |
| 3 | 32.3% | 33.5% | 60.5% |
| 4 | 18.4% | 47.1% | 78.6% |
| 5 | 9.2% | 70.3% | 96.0% |

So 초성 input is a shortcut for long and frequent words and set phrases, with the menu in play;
everyday short words are faster typed in full.

### 15.1 ⇧ + consonant = consonant + ㅏ (owner, 2026-11-07)

**Shift on a consonant key types that consonant with 중성 ㅏ**: ⇧ㅎ is a ㅎ+ㅏ token.

- **In 초성 input it does not end 초성 input.** The token is still a 초성 token, now known to carry
  ㅏ: ㅇㄴ⇧ㅎㅅㅇ reads as ㅇ ㄴ 하 ㅅ ㅇ and still predicts 안녕하세요. Only a vowel key starts
  normal composition. The final stays **open** (owner, 2026-11-07): ⇧ㅎ matches 하, 한, 할, 합 …
- **A consonant after a ⇧ token can be its final** (fix, 2026-11-07): the shown reading is ranked
  with the 초성 predictions. The normal composition (⇧ㄱ ㄴ → 간) wins when it's a known word, and
  ranks high when it's whole syllables; when it leaves loose jamo (ㅇㄴ⇧ㅎㅅㅇ composes ㅇㄴ핫ㅇ),
  the 초성 reading (안녕하세요) shows and the composition is listed.
- **In normal composition** it types the consonant and ㅏ as if both keys were pressed (⇧ㄱ → 가,
  ⇧ㄱ ㄴ → 간), as JIS Moonshot's Shift+consonant does (moonshot.md §9.2).
- Shift on the number and symbol keys stays reserved for digits and symbols (§8.1): a consonant
  that is only on a symbol key has no ⇧ token there. In candidate 2, ㅃ is on R and `;`, so ⇧R is 빠.
- ㅏ is the 중성 of 21% of syllables, so the hint is free often. Marking every ㅏ syllable sharpens
  the 초성 reading a lot (word frequency only):

  | Word length | top-1 plain → with ⇧ㅏ | top-5 plain → with ⇧ㅏ |
  |---|---|---|
  | 2 | 31.6% → 43.7% | 60.0% → 73.0% |
  | 3 | 33.5% → 48.9% | 60.5% → 76.3% |
  | 4 | 47.1% → 65.3% | 78.6% → 91.9% |
  | 5 | 70.3% → 83.9% | 96.0% → 99.2% |

- **[decide]** 초성 input for the other Korean methods (10–12) too, or Bada only.
- **[decide]** the segment: a word up to a space, or the whole preedit (several words: ㄱㅅㅎㄴㄷ →
  감사합니다 spans one word; ㅇㄴㅎㅅㅇ ㅂㄱㅅㄴㄷ spans two).

## 16. Layout 13 is candidate 2 (owner, 2026-11-07)

JIS Bada (id 13) is `docs/layouts/bada-candidate-2.json` with §15 (초성 input) and §15.1 (⇧ + consonant
= consonant + ㅏ). Rules the build adds where the candidate is silent:

- **Compound vowels never join.** Every compound vowel has a key, so a vowel after a vowel always
  starts a new syllable on the implied ㅇ: `ㅂ ㅗ ㅏ` is 보아; ㅘ is the 0 key. **[decide]**: the
  alternative (ㅗ + ㅏ still joins) was offered and not yet chosen.
- **A final moves as its key's initial.** A consonant key typed as a final that a vowel then pulls
  into the next syllable comes back as *that key's* 초성: R typed after 가 is ㅄ; R then ㅏ is 가빠,
  not 갑사. A double final built from two keys gives up its second key.
- **Double finals other than ㅄ** are two consonant keys (ㄴ ㅎ → ㄶ), as in 2-set.
- **The segment** is the whole preedit: Space commits (spec §6.3), so a 초성 string is one word or
  phrase typed without spaces.
- **One consonant alone** shows as itself; predictions start at two 초성 tokens (one consonant is
  how every word starts, and a guessed word there would flash on each).

- **1 is ㅓ** (owner, 2026-11-07), the 중성 alone (with the implied ㅇ: 어); it was a second ㅏ.
  ㅏ stays on 3 and `,`; ㅓ is on `:` too.

### 16.1 F-key actions (owner, 2026-11-07)

Bada publishes these actions (spec §6.5). **No F key is bound to any of them by default.**

| Action id | What it does to the staged input |
|---|---|
| `raw` | Strips all processing: no implied ㅇ, no implied ㅏ, no 종성, no predictions. Every consonant key is its 초성 (R is ㅃ, not ㅄ), every vowel key its vowel, nothing composed: `ㅎ ㅏ ㄴ` → ㅎㅏㄴ. Lasts until the commit |
| `prediction-1` … `prediction-5` | Uses the first … fifth ranked entry (menu index 0 … 4): commits it |
| `qwerty-fullwidth` | The staged keys as JIS QWERTY, full-width (ｈａｎ). Lasts until the commit |
| `qwerty-halfwidth` | The same, half-width (han) |
| `hanja` | Hanja for the **selection**; else for the **staged text**; else a menu of **OECD countries' one-character hanja names** (美 미국, 英 영국, 日 일본 …) |

The OECD list is in `engine/data/oecd.tsv`. **[decide]** several members have no settled
one-character name in Korean usage (Estonia, Latvia, Lithuania, Slovakia, Slovenia, Costa Rica);
the file lists the longer transliterations for them, marked for review.
