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

## Export

| Button | File | Use |
|---|---|---|
| **Export training file** | `bada-learner-export/1` | **Recommended.** An AI agent merges it into your dictionary with you |
| Export as dictionary.json | `bada-dictionary` v1 | To look at, or merge yourself |
| Replace my dictionary… | writes `~/Library/Application Support/bada-ime/dictionary.json` | **Not recommended**: see below |

Exports are also saved in `data/exports/`.

**Training file** (`bada-learner-export/1`):

- `words`: Korean (eojeol), English (lowercased) and Japanese words from the pages' final text, with counts.
- `choseong`: compositions typed with consonant keys only (초성 input) on the first Korean layout
  shown, as `{choseong, word, layout, count, scrolled}`; `scrolled` counts the times you had to move
  through the menu (Tab, ↓, かな) to reach it: the words bada ranks too low.
- `rejected`: committed, then gone from the page: likely wrong readings (`choseong` counts how many
  came from 초성 input).
- `keys`: keystroke counts by key (`⇧`, `⌥` prefixes).
- `texts` (only with "include page texts"): the pages verbatim, for n-gram training.

**Dictionary** (`bada-dictionary` v1): every word used at least *minimum uses* times as a
positive word with its count, and every 초성 → word typed at least that often as a shortcut. No
negative words: deciding what you never mean needs you.

### Why not plug it in directly

It replaces your dictionary with these logs alone: typos you didn't delete become words, hand-made
entries and negatives are dropped, and every shortcut is taken as-is. The old file is kept as
`dictionary.backup-<time>.json` beside it; rename it back to undo.

### Merging with an AI agent (recommended)

In a clone, tell a coding assistant:

```
Read docs/interactive-prompts/learner-merge.md and follow it.
```

It reads the export and your current dictionary (asking first), adds the counts, asks you about
doubtful words, proposes shortcuts and negative words from what you scrolled to and deleted, and
writes the file only when you agree.

## Tips

- Use bada's henkan mode (かな) for Korean, and type as you normally would: the point is your own
  words and your own 초성.
- If keys aren't highlighted or nothing is counted, the browser isn't getting composition events
  from the input method; Safari and Chrome both do.
