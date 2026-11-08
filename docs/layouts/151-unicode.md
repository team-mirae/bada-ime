# Unicode (layout 151)

| Id | Display name | Kind |
|---|---|---|
| 151 | `Unicode` | henkan, search (Mirae Research range 150–199) |

The same mechanism as [Symbols](150-symbols.md), over every named Unicode character.

- **Words** search character names (`snowman` → ☃ ⛇ ⛄); every word must begin a word of the name.
  An exact name comes first, then shorter names. Notes show `U+XXXX name`.
- **A code** gives its character first: hex starting with a digit (`1f600` → 😀) or after `u+` /
  `u` (`u+ac00` → 가).
- Space adds a word, ⇧Space commits and types a space, as in 150.

Data: `engine/data/unicode-names.tsv`, every named character of UnicodeData.txt (algorithmically
named ranges such as CJK ideographs and Hangul syllables, controls, surrogates and private use
left out; their codes still work), built by `engine/tools/build_unicode_data.py`.
