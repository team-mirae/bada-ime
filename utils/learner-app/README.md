# bada learner-app

A local page for learning how *you* type. Pick the layouts to show, type anything in a plain
textarea with bada ime (the real system input method: no emulation), and every keydown, keyup,
composition and input event is logged. Export it as training data, then merge it into your
dictionary ([docs/dictionary-contract.md](../../docs/dictionary-contract.md)).

```sh
cd utils/learner-app
npm start            # or: node server.mjs [--port 5713]
```

Open <http://127.0.0.1:5713/>. Node 20 or later; no packages. `npm test` runs the tests.

## What it logs, and where

`data/sessions/<session>.jsonl`, in this folder, git-ignored, never sent anywhere. **It keeps what
you type, verbatim**, so type nothing you wouldn't keep in a file. Delete `data/` to forget it all.

One JSON object per line: a `meta` line (the layouts shown, the keyboard), then events with `t` in
ms since the session began; the formats are at the top of [lib/learn.mjs](lib/learn.mjs).
The page's text is saved every few seconds, so words are counted from what you kept, not what you
deleted. Changing the layouts shown starts a new session.

The server listens on 127.0.0.1 only and refuses other hosts and cross-site writes.

## Copy practice (`/practice`)

<http://127.0.0.1:5713/practice> shows one sentence at a time to copy with bada ime; press Enter for
the next. It counts every wrong syllable **when it's committed**, even if you fix it afterwards, and
shows what you type instead (리 → 라), the syllables you get wrong most, what you left uncorrected,
accuracy and syllables per minute, per company and over the last 30 sentences.

The sentences are the "II. 사업의 내용" sections of the Samsung group's listed companies' filings
(반기보고서 2026.06), listed in [corpus/samsung-filings.json](corpus/samsung-filings.json) (found with
the Magic DART connector). Download them once, from the public DART viewer:

```sh
node tools/fetch-dart.mjs     # → data/corpus/samsung.json, about 2,000 sentences
```

Results: `data/practice/results.jsonl`, one line per sentence (the target, what you typed, time, keys,
backspaces, the mistakes made). Delete it to start over.

## Export

| Button | File | Use |
|---|---|---|
| **Export training file** | `bada-learner-export/1` | **Recommended.** An AI agent merges it into your dictionary with you |
| Export as dictionary.json | `bada-dictionary` v1 | To look at, or merge yourself |
| Replace my dictionary… | writes `~/Library/Application Support/bada-ime/dictionary.json` | **Not recommended**: see below |

Exports are also saved in `data/exports/`.

**Training file** (`bada-learner-export/1`):

- `words`: Korean (eojeol), English (lowercased) and Japanese words from the pages' final text, with counts.
- `rejected`: committed, then gone from the page: likely wrong readings.
- `keys`: keystroke counts by key (`⇧`, `⌥` prefixes).
- `texts` (only with "include page texts"): the pages verbatim, for n-gram training.

**Dictionary** (`bada-dictionary` v1): every word used at least *minimum uses* times as a
positive word with its count. No negative words: deciding what you never mean needs you.

### Why not plug it in directly

It replaces your dictionary with these logs alone: typos you didn't delete become words, and hand-made
entries and negatives are dropped. The old file is kept as
`dictionary.backup-<time>.json` beside it; rename it back to undo.

### Merging with an AI agent (recommended)

In a clone, tell a coding assistant:

```
Read docs/interactive-prompts/learner-merge.md and follow it.
```

It reads the export and your current dictionary (asking first), adds the counts, asks you about
doubtful words, proposes negative words from what you deleted, and
writes the file only when you agree.

## Tips

- Use bada's henkan mode (かな) for Korean, and type as you normally would: the point is your own
  words.
- If keys aren't highlighted or nothing is counted, the browser isn't getting composition events
  from the input method; Safari and Chrome both do.
