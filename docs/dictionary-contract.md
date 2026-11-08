# Dictionary contract: your own words for bada ime

bada ime ranks what it reads from your keys with built-in word counts. **초성 input** (typing only
initials: ㄱㅅㅎㄴㄷ → 감사합니다) is very ambiguous, so in practice **everyone who types 초성 needs a
dictionary of their own**: the words they mean, the words they never mean, and their personal 초성
shortcuts. This file is that dictionary. It applies to every layout, not only 초성 input.

`docs/dictionary.schema.json` is its JSON Schema; `docs/dictionary-example.json` is a worked
example. [utils/learner-app](../utils/learner-app/README.md) learns entries from your typing. The interactive setup ([interactive-prompts/dictionary.md](interactive-prompts/dictionary.md))
builds one with you.

> Layout 13 is deterministic and no layout types 초성 input at present, so `shortcuts` and the 초성
> matching of positive words are kept but unused for now ([bada-layout.md §8](bada-layout.md)).
> Positive and negative words still rank every layout's readings.

## The file

```json
{
  "format": "bada-dictionary",
  "version": 1,
  "source": { "tool": "Claude Code", "created": "2026-11-07", "description": "from my notes, round 1" },
  "positive": [
    { "word": "데이터센터", "count": 40 },
    { "word": "감사합니다", "confidence": "confirmed" },
    { "word": "kubernetes", "language": "english" },
    { "word": "サーバー", "language": "japanese" }
  ],
  "negative": [
    { "word": "감소합니다", "strength": "strong" },
    { "word": "god", "language": "english" }
  ],
  "shortcuts": [
    { "choseong": "ㄱㅅ", "word": "감사합니다" },
    { "choseong": "ㅇㅋ", "word": "오케이" }
  ]
}
```

**Required:** `format` (`"bada-dictionary"`) and `version` (`1`). Everything else is optional, and
fields bada doesn't know are ignored, so later versions can add some. A file with the wrong format
or version, or that isn't valid JSON, isn't loaded at all; the settings window says why.

### `positive`: words you mean

| Field | | |
|---|---|---|
| `word` | required | The word as you commit it: Hangul for Korean, Latin for English, kana or kanji for Japanese |
| `language` | `korean` (default), `english`, `japanese` | Which readings it counts for |
| `count` | integer ≥ 1 | How often you use it. Added to the built-in count |
| `confidence` | `tentative` 10, `likely` 50 (default), `confirmed` 200 | Used as the count when there's no `count` |

A positive Korean word counts as a known word in every Korean layout, and is offered for its 초성
string (데이터센터 for ㄷㅇㅌㅅㅌ) slightly above built-in words of the same count.

### `negative`: words you never mean

| Field | | |
|---|---|---|
| `word` | required | The reading you don't want |
| `language` | `korean` (default), `english`, `japanese` | Which layouts' readings it applies to (English: QWERTY; Japanese: Kana and Romaji, kanji included) |
| `strength` | `soft` (default), `strong` | `soft`: ranked far down, still in the menu. `strong`: never offered |

### `shortcuts`: your own 초성 abbreviations

| Field | | |
|---|---|---|
| `choseong` | required | The initials exactly as typed (ㄱㅅ). Spaces are ignored |
| `word` | required | What it should give |

A shortcut is offered **first** whenever its exact 초성 string is typed in a layout with 초성 input
(none at present), above every other reading; several for one string keep the file's order.

### `source` (optional)

Who wrote the file and when: shown to no one, kept for you.

### Limits

20,000 positive, 20,000 negative, 5,000 shortcuts; entries past a limit are ignored. Keep the file
under 2 MB.

## Where it goes

`~/Library/Application Support/bada-ime/dictionary.json`. The input menu's **Open Dictionary
Folder** (and the button in bada ime Settings) opens the folder. It's loaded at launch and again
the next time bada is activated in a text field after the file changed (switch away and back).
**Undo:** delete or rename the file; it's dropped on the next activation.

## Privacy

The file holds words, not sentences: no names of people, addresses, account numbers or anything you
wouldn't want in a word list. It stays on your Mac; bada never sends it anywhere.

## The 초성 diagnostics

bada ime can also count, on this Mac, which vowel and final you attach to each initial and which
key you typed it with (spec §8). They hold no words, only counts per key, but they show which
syllables each of your initials really stands for, and how differently you use two keys for the
same jamo (Bada's ㅆ on 2 and on `:`). `tools/analysis/choseong_report.py` summarises them; use it to
decide which words to add, and to check later that the dictionary helped.

## For AI agents

- Build the file **with** the user, from words they name or files they let you read
  ([interactive-prompts/rules.md](interactive-prompts/rules.md)). Read for words, never copy
  sentences, and leave out anything private.
- Prefer real counts (from their writing, or from the diagnostics) over `confidence`.
- **Shortcuts** are for the user's own habits: ask before adding any; never invent them.
- **Negative** words: `soft` unless the user asks for `strong`.
- Show the whole file before writing it, validate it against `docs/dictionary.schema.json`, and ask
  before replacing an existing one (offer a merge).
