# JIS Kana (layout 20)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../images/layouts/20-kana-dark.png">
  <img src="../images/layouts/20-kana-light.png" alt="JIS Kana (layout 20) on an HHKB JIS keyboard">
</picture>

| Id | Display name | Kind |
|---|---|---|
| 20 | `Japanese (Kana)` | henkan, non-deterministic (kanji conversion) |

Legends: bottom of a key unshifted, top with Shift; small grey, the JIS legend. Colours: blue initial or
consonant, orange final, green vowel, purple syllable, maroon kana.

Japanese on the JIS kana legends (JIS Moonshot `A_KANA`), with kanji conversion from Mozc's
dictionary. Full rules in [japanese.md](../japanese.md).

- ⇧ for small kana and punctuation: ⇧3–⇧9 ぁぅぇぉゃゅょ, ⇧0 を, ⇧E ぃ, ⇧Z っ, ⇧, 、 ⇧. 。 ⇧/ ・,
  ⇧[ 「 ⇧] 」.
- ゛ (@) and ゜ ([) attach to the kana before them (か゛ → が, は゜ → ぱ).
- Typing shows the top conversion (わたしはがくせいです → 私は学生です); Tab gives the other
  conversions, ⇧Tab **−1 hiragana**, **−2 katakana**.
- F-key actions: `hiragana`, `katakana`, `halfwidth-katakana`, `fullwidth-latin`, `halfwidth-latin`.

Images: `python3 tools/render/export_png.py` (HHKB JIS). Interactive: open `tools/render/index.html`.
