# JIS Kana Korean (layout 19)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../images/layouts/19-kana-korean-dark.png">
  <img src="../images/layouts/19-kana-korean-light.png" alt="JIS Kana Korean (layout 19) on an HHKB JIS keyboard">
</picture>

| Id | Display name | Kind |
|---|---|---|
| 19 | `Korean (Kana)` | henkan, ranked |

Legends: bottom of a key unshifted, top with Shift; small grey, the JIS legend. Colours: blue initial or
consonant, orange final, green vowel, purple syllable, maroon kana.

Korean on the JIS kana layout: each kana key types its Korean approximation (た 타, し 시, ん
final ㄴ), so a kana typist types Korean without learning a second layout. Full rules in
[kana-korean.md](../kana-korean.md).

- ⇧3–⇧0 (small kana) change the previous vowel: き ⇧ゃ → 캬.
- ゛ voices the previous initial (카゛ → 가); ゜ makes ㅎ ㅍ.
- ⇧ん final ㅇ, ⇧む final ㅁ, ⇧の 의; ⇧つ (っ) makes the next initial tense.
- The table's reading shows; other pronunciations that spell known words are offered
  (はんぐく → 한국, そうる → 서울).

Images: `python3 tools/render/export_png.py` (HHKB JIS). Interactive: open `tools/render/index.html`.
