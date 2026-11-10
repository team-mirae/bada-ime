# Merge learner-app data into your dictionary

The user typed in [utils/learner-app](../../utils/learner-app/README.md) and exported a training
file (`bada-learner-export/1`). Merge it into their `dictionary.json` with them. Follow
[rules.md](rules.md): you're reading their own writing.

Read first: [docs/dictionary-contract.md](../dictionary-contract.md) (its **For AI agents**
section in full) and the export format in the learner-app README.

## 1. Find the files

1. > Which training file? The newest is in `utils/learner-app/data/exports/`.
   Ask before listing or reading it (rules §1). If it has `texts`, say so and don't quote them back;
   use them only for counting.
2. > May I read your current dictionary, `~/Library/Application Support/bada-ime/dictionary.json`?
   If there's none, you'll build a new one. If it doesn't parse or isn't `bada-dictionary` v1, show
   the error and ask: fix it first, or start a new one beside it.

Say in a few lines what's there: sessions and keystrokes, words per language, rejected rows; and what the dictionary has now (counts per section).

## 2. Positive words

- Add each export word's `count` to the same word's `count` in the dictionary (same `language`);
  an entry with only `confidence` keeps it and gains the count.
- Words used once, and words that look like typos (an existing word with one syllable or letter
  off, a stray jamo), are **doubtful**: show them, at most about 30 at a time, and ask which to keep.
- Leave out names of people, addresses, numbers that look like accounts or phone numbers, and
  anything private, even when it was typed often; ask if unsure.
- Stay within the contract's limits; if over, keep the highest counts and say what was dropped.

## 3. Negative words

From `rejected`: words committed and then deleted. Most are slips, not words to ban. Show the ones
deleted more than once, and ask about each group:

> These came out and you deleted them. Any you never mean? (soft: ranked far down; strong: never shown)

Never add a negative word the user didn't confirm; never make a positive word negative.

## 4. Write it

1. Show what changes: words added, counts raised, negative words added, anything
   dropped. Then the full JSON if they want it.
2. Validate against [dictionary.schema.json](../dictionary.schema.json) (a few lines of Python
   without packages will do).
3. Writing is a change (rules §3): ask, then copy the old file to
   `dictionary.backup-<date>.json` beside it and write the new one. Update `source` (tool, date, "merged learner-app export").
4. It loads the next time bada is activated (switch away and back); bada ime Settings shows what loaded.

## 5. Wrap up

Say what changed, where the backup is, and how to undo it (rename the backup back). Suggest typing
in the learner-app again after a while: each export refines the dictionary.
