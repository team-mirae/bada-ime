# JIS Kana (20) and JIS Romaji (21)

Status: built in v0.1 (owner, 2026-11-07), with **kanji conversion** since 2026-11-07. Code: `engine/src/japanese.rs`; script forms and F-key actions in
`engine/src/engine.rs`.

## 20 JIS Kana

Every key types its JIS kana legend (the table in jis-moonshot `docs/moonshot.md` §3), with Shift
for small kana and punctuation: ⇧3–⇧9 ぁぅぇぉゃゅょ, ⇧0 を, ⇧E ぃ, ⇧Z っ, ⇧, 、, ⇧. 。, ⇧/ ・,
⇧[ 「, ⇧] 」. ゛ (@) and ゜ ([) attach to the kana before them (か゛ → が, は゜ → ぱ, う゛ → ゔ) and stay
as they are where they can't.

## 21 JIS Romaji

The common Japanese IME spellings (`ka` か, `shi` / `si` し, `chi` / `ti` ち, `tsu` / `tu` つ, `fu` /
`hu` ふ, `ja` / `jya` / `zya` じゃ, `kya` きゃ, `fa` ふぁ, `thi` てぃ, `va` ゔぁ, small `x…` / `l…`, `xtu` っ):

- a doubled consonant is っ (`gakkou` がっこう), and so is `tch` (`matcha` まっちゃ);
- `n` before a consonant is ん; `nn` is ん, and a vowel after it starts the next kana on the second
  n (`konnichiha` and `konnnichiha` both こんにちは; `kinnyoubi` きんようび); `n'` is ん (`tan'i`
  たんい); a trailing `n` reads as ん;
- `-` ー, `,` 、, `.` 。, `/` ・, `[` 「, `]` 」; digits and other symbols full-width;
- ⇧ + a letter is that capital, full-width, as it is (Japanese IMEs switch to Latin there);
- letters still pending (`ky`) show as they are.

## Script forms (spec §6.4)

Both can offer the negative menu entries: **−1 hiragana, −2 katakana** of their reading.
**Off by default**: Settings → "Script forms below the top entry" turns them on. Only one method
contributes (settings: "Negative menu entries"); by default the lowest enabled of 20 and 21. Off,
katakana is an ordinary ranked entry and the kana a conversion row (see Kanji conversion).

## F-key actions (spec §6.5)

Both publish `hiragana`, `katakana`, `halfwidth-katakana`, `fullwidth-latin` and `halfwidth-latin`
(Latin is the keys as JIS QWERTY: for Romaji the letters typed). Nothing is bound by default; the
settings window lists them under each method. Each lasts until the commit.

## Ranking

A reading that is a known word (`engine/data/ja-words.txt`, JIS Moonshot's hand-written list)
scores above an unknown Korean composition; an unknown one below it; one with a stray ゛ ゜ or
pending Latin letters low. So with 13 and 21 enabled, `konnichiha` shows こんにちは and ㄱㅅㅎㄴㄷ
still shows 감사합니다.

## Kanji conversion

Both layouts convert their kana reading to kanji (`engine/src/kanji.rs`, ported from JIS Moonshot:
a word lattice over the kana, word costs plus part-of-speech connection costs, at most 40 kana).
The data is `engine/data/kanji.bin` (11 MB), built by `engine/tools/kanji/build_kanji.py` from
**Mozc's open-source dictionary** (pinned commit); notices in `engine/data/LICENSE-mozc.txt`.
On Mozc's evaluation set (`engine/tests/data/kanji-evaluation.tsv`): 69% exact, 79% in the top 3.

- The menu (spec §6.4) is **0 and forward: the conversions, cheapest first, with the kana among
  them**; with script forms on, also **−1 hiragana, −2 katakana**. So typing shows the top
  conversion (わたしはがくせいです → 私は学生です), Tab the other conversions.
- **Where the kana goes** (owner, 2026-11-08): where conversion ranks it; a known word's kana
  (`ja-words.txt`, or kana-native) no lower than second (あなた: 貴方 may come first, あなた next); a
  **kana-native** word's kana first. Otherwise last.
- **Kana-native words** (`engine/data/ja-kana-native.txt`, built into the engine): about 870 words
  written in kana at least 60% of the time in Tatoeba's Japanese sentences (この, その, こと, もの,
  あなた, ちょっと, ありがとう), counted by `engine/tools/build_kana_native.py` with the robot's test
  sentences left out. Conversion treats their kana as one below their cheapest kanji, so phrases
  keep them in kana too (このほん → この本, not 個の本). The cost: Mozc's evaluation set lost 5 exact
  sentences of 613 (431 → 426).
- Up to six conversions; one that is just the kana isn't repeated, nor its katakana with script
  forms on.
- Unknown or broken readings (a stray ゛, pending Latin letters) aren't converted.

## Menu notes (owner, 2026-11-07)

Each Japanese row carries a gray note at its right: **kana rows their romaji** (Hepburn as IMEs spell
it: らーめん ra-men, がっこう gakkou, しんよう shin'you; `engine/src/romaji.rs`), **kanji rows their
kana reading** (私は学生です わたしはがくせいです). Japanese rows are drawn in Hiragino Sans, so kanji
take their Japanese forms.
