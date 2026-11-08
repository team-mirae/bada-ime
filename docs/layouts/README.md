# Layouts

Every layout bada ime builds, one page each, and [combinations](combinations.md) of them
enabled together. Ids, kinds and names are the [spec's catalog](../spec.md) (§5).

| Id | Layout | Display name | Kind |
|---|---|---|---|
| 0 | [JIS QWERTY](00-qwerty.md) | `QWERTY` | muhenkan |
| 10 | [JIS KTS](10-kts.md) | `Korean (2-set)` | henkan, deterministic |
| 11 | [JIS Korean Legacy](11-korean-legacy.md) | `Korean (legacy-korean)` | henkan, deterministic |
| 12 | [JIS Moonshot](12-moonshot.md) | `Korean (legacy-moonshot)` | henkan, deterministic |
| 13 | [JIS Bada](13-bada.md) | `Korean (bada)` | henkan, deterministic |
| 14 | [JIS 3-set](14-3set-jis.md) | `Korean (3-set JIS)` | henkan, deterministic |
| 18 | [JIS Korean Roman](18-korean-roman.md) | `Korean (Roman)` | henkan, ranked |
| 19 | [JIS Kana Korean](19-kana-korean.md) | `Korean (Kana)` | henkan, ranked |
| 20 | [JIS Kana](20-kana.md) | `Japanese (Kana)` | henkan, non-deterministic (kanji conversion) |
| 21 | [JIS Romaji](21-romaji.md) | `Japanese (Romaji)` | henkan, non-deterministic (kanji conversion) |
| 150 | [Symbols](150-symbols.md) | `Symbols` | henkan, search |
| 151 | [Unicode](151-unicode.md) | `Unicode` | henkan, search |

Examples of layouts enabled together: [QWERTY + bada + kana](combinations.md#qwerty--bada--kana),
[bada + romaji](combinations.md#bada--romaji), [QWERTY + romaji](combinations.md#qwerty--romaji),
[QWERTY + KTS](combinations.md#qwerty--kts).

Every layout has a key map, `NN-name.json`, in the layout editor's format (`bada-layout/1`). The
engine reads `14-3set-jis.json` as it is; `13-bada.json` and `19-kana-korean.json` are the
references the engine and the renderer follow; the others are written from the engine by
`cargo run --example layout_json` (in `engine/`). Each file also says what the [typing robot](../robot.md) needs:

- `languages`: what the layout types, from `english`, `korean`, `japanese_kana`, `japanese_romaji`
  (usually one; none for the searches 150 and 151);
- `reliability`: for each language in `languages`, what the layout claims: `perfect`, `production`
  or `experimental`. The typing robot holds it to the claim, in CI ([robot.md](../robot.md#reliability));
- `input`: how: `direct` (a key, a character), `compose` (jamo into syllables), `roman`, `kana`,
  `romaji` (spelled, then converted), `transcription` (19), `search`;
- `rules` (compose): `join_vowels`, `implied_ieung`, `double_toggle`, `held_shift`, `double_tense`;
- `spelling` (`roman`, `romaji`): the spelling the robot types for each jamo or kana (21: `ん'` is ん
  before a vowel, y or n, `n'`);
- `transcription` (19): the tenseness series, what ゛ ゜ っ do to an initial, small vowels, ー, and
  the readings the menu offers (vowels, finals).

Token roles: `i` initial, `f` final, `c` initial or final, `v` vowel, `a` initial with ㅏ, `n` vowel
on ㅇ, `k` kana, `l` letter, `x` text. Two tokens on one key are alternatives, chosen by the
composer's state (13's `]`: ㅃ or ㅄ). `cargo test --test robot` checks every file against the engine.

Images: `python3 tools/render/export_png.py` (HHKB JIS). Interactive: open `tools/render/index.html`.

## Layout metadata and maintainers

Every layout page (`NNN-name.md`) opens with a metadata header, in whatever form its maintainer
likes (the pages here use a table), giving:

- **Name:** the full canonical name in English, Korean, Japanese and the layout's native script
  language (more languages if the maintainer wants them);
- **Short name:** a short human-readable name in the same languages;
- **In-code name:** unique among all layouts (the pages here use a lowercase slug, with the id and
  the engine's constant);
- **Abbreviations** and shorthands (KTS for Korean 2-set);
- **Designer:** the original designer, an organization or a person;
- **Maintainer:** who maintains it now (may be the designer);
- **Contact:** a working email for the maintainer, and optionally more.

**Ownership.** The current maintainer owns the design decisions of their layout. If they don't
respond within **90 days** to an email from an `@sjun.me` address or a repository admin, ownership
passes to **Mirae Research**.

The layouts in this repository as of 0.1.0 are maintained by Mirae Research, designer Min Jun Sung;
where a layout builds on an earlier design (QWERTY, KS X 5002, JIS kana, 세벌식), its page names that
design too.

