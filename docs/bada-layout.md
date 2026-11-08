# JIS Bada (layout 13)

bada ime's own Korean layout, for JIS keyboards. It's **deterministic**: the same keys always give
the same text, nothing is guessed. The page with the board image is
[layouts/13-bada.md](layouts/13-bada.md); the key map's source is `docs/layouts/13-bada.json`; the
engine is `engine/src/layout.rs` (keys and composing) and `engine/src/typo.rs` (typo suggestions).
How it came to be: [bada-history.md](bada-history.md).

## 1. What it is

**bada types like 2-set, keeps finals like 3-set, and uses the whole key space like JIS kana.**
(두벌식처럼 치고, 세벌식처럼 받침을 지키고, JIS 가나처럼 키보드 전체를 쓴다.) It takes how each works,
not where its keys are: KTS habits don't carry over key for key.

| From | What bada takes | What it doesn't |
|---|---|---|
| **KTS (2-set)** | Consonants on the left, vowels on the right, so the hands alternate. Consonant keys that are a final until a vowel takes them. Vowels that join (ㅗ ㅏ → ㅘ). | KTS's key positions. Shift for tense consonants and ㅒ ㅖ. |
| **3-set (세벌식)** | Final-only keys (J K L ; ¥ and their ⇧ double finals) that never move to the next syllable. Every compound vowel on a key of its own. | 3-set's sides (initials right, finals left there). A final-only key for every final. |
| **JIS kana** | All 48 kana-space keys carry letters, the number row included, and ⇧ on number and symbol keys is kept for digits and symbols, as JIS kana keeps it for its punctuation. | Anything kana-specific. |

Its aims: every common jamo on an unshifted key, Korean typed without moving the wrist off the
home position, and digits and punctuation typed in the middle of Korean with no mode switch and no
prediction. 英数 and かな, under the thumbs either side of the space bar, change language.

## 2. Keys

Unshifted on the first line of each row, ⇧ on the second (empty: as unshifted). ᶠ marks a
final-only key; a consonant without it is an initial, or a final until a vowel takes it (§3).

| | `1` | `2` | `3` | `4` | `5` | `6` | `7` | `8` | `9` | `0` | `-` | `^` | `¥` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| key | ㅒ | ㅆ | ㅉ | ㄲ | ㄸ | ㅟ | ㅝ | ㅠ | ㅛ | ㅘ | ㅙ | ㅞ | ㅂᶠ |
| ⇧ | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 0 | - | ^ | ㄼᶠ |

| | `Q` | `W` | `E` | `R` | `T` | `Y` | `U` | `I` | `O` | `P` | `@` | `[` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| key | ㅃ ㅄᶠ | ㅌ | ㄴ | ㅅ | ㅍ | ㅚ | ㅕ | ㅑ | ㅔ | ㅐ | ㅖ | ㅒ |
| ⇧ |  |  |  |  |  |  |  |  |  |  | @ | [ |

| | `A` | `S` | `D` | `F` | `G` | `H` | `J` | `K` | `L` | `;` | `:` | `]` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| key | ㄷ | ㅈ | ㄱ | ㅇ | ㅁ | ㅏ | ㅇᶠ | ㄴᶠ | ㄹᶠ | ㄱᶠ | ㅆ | ㅃ ㅄᶠ |
| ⇧ |  |  |  |  |  |  | ㅁᶠ | ㄶᶠ | ㄺᶠ | ㄲᶠ | : | ] |

| | `Z` | `X` | `C` | `V` | `B` | `N` | `M` | `,` | `.` | `/` | `\` |
|---|---|---|---|---|---|---|---|---|---|---|---|
| key | ㅊ | ㅋ | ㄹ | ㅎ | ㅂ | ㅓ | ㅣ | ㅡ | ㅗ | ㅜ | ㅢ |
| ⇧ |  |  |  |  |  |  |  | , | . | / | \ |

- **Initial-only:** F ㅇ, 3 ㅉ, 5 ㄸ (ㅉ and ㄸ are never finals).
- **Interlaced:** Q and `]` type ㅃ as an initial and ㅄ as a final, by the composer's state (after a
  vowel: ㅄ, 값; otherwise ㅃ), not by Shift. A vowel after ㅄ takes ㅃ: 값 ㅏ → 가빠.
- **Initial-or-final:** W E R T, A S D G, Z X C V B, 2 4 and `:`.
- **Final-only:** J ㅇ, K ㄴ, L ㄹ, `;` ㄱ, ¥ ㅂ, the five commonest finals; with ⇧, ㅁ ㄶ ㄺ ㄲ ㄼ.
- **Vowels:** all 21 have a key: the simple ones on the right half and the bottom row, the compound
  ones and ㅒ ㅖ on the number row and the keys right of P.

## 3. Composing

- **ㅇ is typed** (F). A vowel with nothing before it is the bare vowel (ㅏ).
- **A consonant after a vowel is its final.** An initial-or-final key stays the final until the
  next key is a vowel, which takes it as its initial, as in 2-set: 가 E → 간, 가 E ㅏ → 가나.
- **A final-only key never moves:** 가 K ㅏ → 간ㅏ. Use E for a ㄴ that starts the next syllable.
- **Double finals:** two finals in a row join when they make one (ㄹ ㅁ → ㄻ, ㄴ ㅎ → ㄶ); a vowel
  then takes the second as its initial if its key can be one. Or one ⇧ key: ⇧J ㅁ, ⇧K ㄶ, ⇧L ㄺ,
  ⇧; ㄲ, ⇧¥ ㄼ.
- **Vowels join as in 2-set** (ㅗ ㅏ → ㅘ, ㅡ ㅣ → ㅢ), though every compound vowel has its key too.
- **Consonants alone stay as typed** (ㄱㅅ, ㅋㅋ): there is no 초성 input.

Examples: 한 V H K · 안녕 F H K E U J · 것이 D N R F M · 많다 G H ⇧K A H ·
감사합니다 D H G R H V H B E M A H.

## 4. Shift, digits and symbols

No jamo sits on ⇧ of a number or symbol key, except ⇧¥ (ㄼ) and ⇧; (ㄲ). So digits and punctuation
are typed directly in henkan mode: nothing to predict, nothing to switch.

| Keys | Type |
|---|---|
| ⇧ + a letter key | the unshifted jamo, unless the key has a ⇧ final (J K L) |
| ⇧ + a number or symbol key | its own JIS character: 1–0 - ^ @ [ : ] , . / \ |
| ⇧⌥ + a number or symbol key | its shifted JIS character: ! " # $ % & ' ( ) = ~ \| ` { + * } < > ? _ |
| not in henkan | ¥ and ; (⇧ on their keys is ㄼ and ㄲ): type them in muhenkan (英数) |

On a US ANSI keyboard the ¥ and ろ keys are missing; everything on them can still be typed: final ㅂ
with B, ㄼ with L B, ㅢ with , M.

## 5. Typo suggestions

What was typed always shows first. Readings one slip away are suggested under it, marked `typo?`
in the menu's note column; they never replace it, nor whatever ranks first among the enabled
layouts.

- **Language model:** a jamo 4-gram model (initial, vowel and final are distinct symbols) trained on
  `engine/data/ko-words.tsv`, each word weighted by its count, Witten–Bell interpolated:
  `engine/data/ko-jamo-4gram.bin` (149k n-grams, 1.4 MB), built by `engine/tools/build_jamo_lm.py`.
- **Slips** (one per reading; log10 cost):

  | Slip | Cost |
  |---|---|
  | D typed for the final ㅇ (J) | −0.7 |
  | the same consonant on its other key, e.g. K (final-only ㄴ) for E | −1.3 |
  | Shift missed or extra on a key whose ⇧ is another jamo | −2.0 |
  | **NKRO typing**: two neighbouring keys on different hands registered out of order | −0.8 |
  | NKRO typing: two neighbouring keys on one hand (different fingers) out of order | −1.3 |
  | NKRO typing: three keys of a roll reordered (all different fingers) | −1.8 |
  | the next key on the same row | −2.5 |

  NKRO typing is a fast roll whose keys register in the wrong order. Keys on one finger can't
  overlap, so they're never taken as reordered (fingers: touch typing on JIS, `typo.rs`).
  Example: ㅎ ㄴ ㅏ (V K H) suggests 한; 가맛합니다 (ㅏ and ㅅ rolled) suggests 감사합니다.

- **Shown** when its probability times its slip's is at least a tenth of the typed reading's (log10
  ratio ≥ −1.0), it isn't the same text, and it has fewer loose jamo than what was typed (or none).
  At most three; the user's strong negative words are left out.
- **Measured** (`cargo run --release --example typo_eval`: 3,000 common words drawn by √count, one
  slip each, drawn by cost): the intended word is suggested for **98.5%** of slipped words (96.7%
  first); **5.2%** of correctly typed words get a suggestion.

## 6. F-key actions

Bound to nothing by default (spec §6.5).

| Action id | What it does to the staged input |
|---|---|
| `raw` | Every key as its jamo, nothing composed: `ㅎ ㅏ ㄴ` → ㅎㅏㄴ. Lasts until the commit |
| `prediction-1` … `prediction-5` | Commits the first … fifth menu entry (index 0 … 4) |
| `qwerty-fullwidth` | The staged keys as JIS QWERTY, full-width (ｖｈｋ). Lasts until the commit |
| `qwerty-halfwidth` | The same, half-width (vhk) |
| `hanja` | Hanja for the **selection**; else for the **staged text**; else a menu of **OECD countries' one-character hanja names** (美 미국, 英 영국, 日 일본 …) |

The OECD list is `engine/data/oecd.tsv`. **[decide]** several members have no settled one-character
name in Korean usage (Estonia, Latvia, Lithuania, Slovakia, Slovenia, Costa Rica); the file lists
the longer transliterations for them, marked for review.

## 7. Tools

- **Board images and the interactive renderer:** `tools/render` (`python3 tools/render/build_layouts.py`
  after changing `13-bada.json`, then `export_png.py`).
- **Key drill:** `tools/practice/index.html`. **Your own typing, logged and exported:**
  `utils/learner-app`.

## 8. Open questions

1. **[decide]** ⌥¥ and ⌥; (no Shift) typing ¥ and ;, so that every JIS character can be typed in henkan.
2. **[decide]** 초성 input on request: a key that turns typed consonants into a word menu. The engine
   still has the machinery (prediction, the dictionary's shortcuts, `choseong` diagnostics), unused.
3. **[decide]** learning the slips' costs (§5) from real typing (the learner-app logs backspaced keys).
