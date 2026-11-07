# Layouts

Every layout bada ime builds, one page each, and [combinations](combinations.md) of them
enabled together. Ids, kinds and names are the [spec's catalog](../spec.md) (§5).

| Id | Layout | Display name | Kind |
|---|---|---|---|
| 0 | [JIS QWERTY](00-qwerty.md) | `QWERTY` | muhenkan |
| 10 | [JIS KTS](10-kts.md) | `Korean (2-set)` | henkan, deterministic |
| 11 | [JIS Korean Legacy](11-korean-legacy.md) | `Korean (legacy-korean)` | henkan, deterministic |
| 12 | [JIS Moonshot](12-moonshot.md) | `Korean (legacy-moonshot)` | henkan, deterministic |
| 13 | [JIS Bada](13-bada.md) | `Korean (bada)` | henkan, ranked |
| 18 | [JIS Korean Roman](18-korean-roman.md) | `Korean (Roman)` | henkan, ranked |
| 19 | [JIS Kana Korean](19-kana-korean.md) | `Korean (Kana)` | henkan, ranked |
| 20 | [JIS Kana](20-kana.md) | `Japanese (Kana)` | henkan, non-deterministic (kanji conversion) |
| 21 | [JIS Romaji](21-romaji.md) | `Japanese (Romaji)` | henkan, non-deterministic (kanji conversion) |

Examples of layouts enabled together: [QWERTY + bada + kana](combinations.md#qwerty--bada--kana),
[bada + romaji](combinations.md#bada--romaji), [QWERTY + romaji](combinations.md#qwerty--romaji),
[QWERTY + KTS](combinations.md#qwerty--kts).

The JSON files here are saved layout candidates: `bada-candidate-1.json`, `bada-candidate-2.json`
(layout 13) and `kana-korean.json` (layout 19).

Images: `python3 tools/render/export_png.py` (HHKB JIS). Interactive: open `tools/render/index.html`.
