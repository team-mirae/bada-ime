# JIS Bada (layout 13)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../images/layouts/13-bada-dark.png">
  <img src="../images/layouts/13-bada-light.png" alt="JIS Bada (layout 13) on an HHKB JIS keyboard">
</picture>

| Id | Display name | Kind |
|---|---|---|
| 13 | `Korean (bada)` | henkan, ranked |

Legends: bottom of a key unshifted, top with Shift; small grey, the JIS legend. Colours: blue initial or
consonant, orange final, green vowel, purple syllable, maroon kana.

The new layout this IME is named for: every jamo on an unshifted key across all 48 kana-space
keys, so Korean needs no Shift. Designed from corpus measurements in
[bada-layout.md](../bada-layout.md); the build is candidate 2 (§16).

- **Every vowel, compound ones too, has a key** (the number row and the symbol keys carry most of
  them), and compound vowels never join: a vowel after a vowel starts a new syllable.
- **Implied ㅇ**: a vowel with no consonant before it gets ㅇ (ㅓ → 어).
- **⇧ + consonant = consonant + ㅏ** (⇧ㅎ → 하). ⇧ on a number or symbol key types its JIS character;
  ⇧⌥ its shifted JIS character (⇧⌥/ → ?).
- **초성 input**: consonants alone are read as a 초성 string and ranked against known words
  (ㄱㅅㅎㄴㄷ → 감사합니다); ⇧ consonants keep it going and sharpen it (§15).
- **Token keys**: R and `;` both carry ㅃ / ㅄ, read as an initial or a final by position. A final
  pulled into the next syllable comes back as its key's initial (R then ㅏ: 가빠).
- F-key actions (bound to nothing by default): `raw`, `prediction-1`…`5`, `qwerty-fullwidth`,
  `qwerty-halfwidth`, `hanja` (§16.1).

Bada is the default henkan candidate. Its usual companions are QWERTY and kana
([QWERTY + bada + kana](combinations.md#qwerty--bada--kana)) or romaji
([bada + romaji](combinations.md#bada--romaji)).

Images: `python3 tools/render/export_png.py` (HHKB JIS). Interactive: open `tools/render/index.html`.
