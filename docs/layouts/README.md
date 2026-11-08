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

Key maps in the layout editor's JSON: `13-bada.json` (layout 13; the engine and the renderer follow
it), `14-3set-jis.json` (layout 14; the engine reads it as it is) and `kana-korean.json` (layout 19).

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

