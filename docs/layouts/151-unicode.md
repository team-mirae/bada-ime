# Unicode (layout 151)

<!-- layout metadata -->
| | |
|---|---|
| **Name** | English: Unicode · 한국어: 유니코드 · 日本語: ユニコード |
| **Native script** | (none: every script) |
| **Short name** | English: Unicode · 한국어: 유니코드 · 日本語: ユニコード |
| **In-code name** | `unicode` (id 151, `layout::UNICODE`) |
| **Reliability** | none: a search, it declares no language |
| **Abbreviations** | — |
| **Designer** | Min Jun Sung (Mirae Research). Character names: the Unicode Character Database |
| **Maintainer** | Mirae Research (미래 리서치) |
| **Contact** | mj@teammirae.com (Mirae Research); issues at https://github.com/team-mirae/bada-ime |

| Id | Display name | Kind |
|---|---|---|
| 151 | `Unicode` | henkan, search (Mirae Research range 150–199) |

The same mechanism as [Symbols](150-symbols.md), over every named Unicode character.

- **Words** search character names (`snowman` → ☃ ⛇ ⛄); every word must begin a word of the name.
  An exact name comes first, then shorter names. Notes show `U+XXXX name`.
- **A code** gives its character first: hex starting with a digit (`1f600` → 😀) or after `u+` /
  `u` (`u+ac00` → 가).
- **Colloquial names** (Korean and Japanese, romanized, several spellings, with English glosses)
  come before Unicode's own names: `ju` / `joo` / `corporation` → ㈜, `kabu` → ㈱, `pyeong` /
  `pyung` → 평 坪, `yuubin` → 〒, `reiwa` → ㋿, `giyeok` → ㉠ ㈀. An exact canonical name first, then
  a tag typed whole, then the rest, ties in the file's order. They're Unicode's only (not Symbols'),
  so a profile with both layouts lists each once.
- Space adds a word, ⇧Space commits and types a space, as in 150.

Data: `engine/data/unicode-aliases.tsv` (hand-written: character, canonical colloquial name, tags),
merged into `engine/data/unicode-names.tsv`, every named character of UnicodeData.txt (algorithmically
named ranges such as CJK ideographs and Hangul syllables, controls, surrogates and private use
left out; their codes still work), built by `engine/tools/build_unicode_data.py`.
