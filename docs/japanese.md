# JIS Kana (20) and JIS Romaji (21)

Status: built in v0.1, **deterministic** (owner, 2026-11-07): each reads the keys as kana, and
nothing converts to kanji yet. Code: `engine/src/japanese.rs`; script forms and F-key actions in
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

Both offer the negative menu entries: **−1 hiragana, −2 katakana** of their reading. Only one method
contributes (settings: "Negative menu entries"); by default the lowest enabled of 20 and 21.

## F-key actions (spec §6.5)

Both publish `hiragana`, `katakana`, `halfwidth-katakana`, `fullwidth-latin` and `halfwidth-latin`
(Latin is the keys as JIS QWERTY: for Romaji the letters typed). Nothing is bound by default; the
settings window lists them under each method. Each lasts until the commit.

## Ranking

A reading that is a known word (`engine/data/ja-words.txt`, JIS Moonshot's hand-written list)
scores above an unknown Korean composition; an unknown one below it; one with a stray ゛ ゜ or
pending Latin letters low. So with 13 and 21 enabled, `konnichiha` shows こんにちは and ㄱㅅㅎㄴㄷ
still shows 감사합니다.

## Not yet

- Kanji conversion (JIS Moonshot's Mozc dictionary, `kanji.bin`), and so non-deterministic
  ranking for Japanese.
