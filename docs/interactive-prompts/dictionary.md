# Build your bada ime dictionary (strongly recommended)

You come here from [start.md](start.md), or directly with bada ime installed. Follow
[rules.md](rules.md); here it matters most, since you may be reading the user's own writing.

Read [docs/dictionary-contract.md](../dictionary-contract.md) first, its **For AI agents** section in
full. The file you'll write is validated by [dictionary.schema.json](../dictionary.schema.json);
[dictionary-example.json](../dictionary-example.json) is a worked example.

## Why

Say, in your own words:

> bada ime guesses words from very little: in 초성 input, ㄱㅅ could be 감사, 가사, 기사, 검사… Its
> built-in word counts are generic. Your dictionary tells it the words you mean, the ones you never
> mean, and your own 초성 shortcuts, so the right one comes first. It stays on this Mac. About fifteen
> minutes.

> Shall we build it now?
>
> 1. Yes (recommended)
> 2. Later: tell me where it goes and how to start
> 3. No

- **2:** say where it goes (`~/Library/Application Support/bada-ime/dictionary.json`, opened by the
  input menu's "Open Dictionary Folder"), that the contract explains it, and that saying "Read
  docs/interactive-prompts/dictionary.md and follow it" in the clone starts this again. Finish.
- **3:** finish; don't bring it up again.

## 1. What they type

One at a time:

1. > Which languages do you type, and do you type 초성 (initials only) for whole words?
2. > Is there writing of yours I may read to find your words: notes, documents, a word list, chat
   > you wrote yourself? You choose the files; I only read what you name, and only words go into
   > the file.
   >
   > 1. Yes, I'll name files or a folder
   > 2. No, I'll tell you the words myself
   > 3. No words: just the questions
3. If they type 초성 and diagnostics are on: > May I read bada ime's 초성 diagnostics? They're counts
   of which syllables your initials became, per key; no words or text.
   If yes (a read, rules §1), run `python3 tools/analysis/choseong_report.py` and tell them, in a few
   lines, what stands out (for example: "your ㅅ on S is mostly 사, on O mostly 서").

For files: ask before each one. Read for words, not content: don't quote their text back, and never
put sentences, people's names, addresses, account numbers or anything private in the file. If a file
looks private (mail, other people's messages, finances), ask before using anything from it.

## 2. Positive words

From what step 1 gave you, build `positive`:

- **Korean:** words they use that generic lists may miss or rank low: jargon, compounds
  (데이터센터), set phrases and endings they type whole (확인했습니다). For 초성 typists, every phrase
  they abbreviate.
- **English** and **Japanese** words they mix in (Japanese as they commit it: kana or kanji).
- Real `count`s when you counted them; else `confidence`.

Show the list grouped by language, at most about 30 per group on screen, and ask them to strike
anything wrong.

## 3. Shortcuts (초성 typists)

> Which phrases do you type as 초성 every day? Tell me the initials and what they should give,
> e.g. ㄱㅅ → 감사합니다.

Only theirs: never invent shortcuts. Several for one 초성 string keep the order they give.

## 4. Negative words

Ask, one at a time, only what fits:

1. > Which words keep coming out first that you never mean? (For 초성: what does ㄱㅅ give that you
   > never want?)
2. (Mixed typists) > English or Japanese readings that win where you meant Korean?

Each goes in `negative` with its `language`; `soft` unless they ask for `strong` ("never show it").

## 5. Write it

1. Assemble the JSON (`"format": "bada-dictionary"`, `"version": 1`, a `source` naming you and the
   date). Show it in full.
2. Validate it against the schema. A few lines of Python with no third-party packages, checking the
   required fields, types and limits, will do; installing a validator is its own question.
3. Writing it is a change (rules §3): ask, then save it as
   `~/Library/Application Support/bada-ime/dictionary.json`. If one is there, show what differs and
   ask: replace, merge, or keep.
4. It loads the next time bada is activated in a text field (switch away and back). bada ime
   Settings shows what loaded, or why it didn't.

## 6. Try it

Ask them to type a few of their 초성 strings and words, and say what came first. For one still wrong:
a positive word, a shortcut, or a negative word, then write the file again.

## 7. Wrap up

Say what's in the file (counts per section), where it is, and how to undo it: delete or rename
`dictionary.json`. Remind them the 초성 diagnostics keep counting (Settings → 초성 diagnostics → Clear
or turn off), so running this prompt again later can refine the dictionary.
