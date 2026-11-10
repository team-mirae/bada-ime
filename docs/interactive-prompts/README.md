# Interactive setup prompts

Prompts for an AI assistant that walk someone through installing **bada ime** and then building
their **dictionary** (recommended: your own words, ranked first).

## Start

In a coding assistant that can run commands (Claude Code, for example), started in a clone of this
repository, say:

```
Read docs/interactive-prompts/start.md and follow it.
```

Already installed? Go straight to the dictionary:

```
Read docs/interactive-prompts/dictionary.md and follow it.
```

In a chat assistant without access to your computer, paste [rules.md](rules.md), then the prompt.
It can't run anything, so it tells you what to check and type yourself.

## Files

| File | What |
|---|---|
| [rules.md](rules.md) | Rules for every prompt: ask before reading anything, before installing, before changing settings |
| [start.md](start.md) | Check the clone, build, install, set up the input sources and layouts. Ends with dictionary.md |
| [dictionary.md](dictionary.md) | Build `dictionary.json` with the user: positive and negative words |
| [learner-merge.md](learner-merge.md) | Merge a [learner-app](../../utils/learner-app/README.md) export into `dictionary.json` |

## Writing a prompt

- Write it to the assistant ("Ask…", "Don't…"), with the questions for the user as quotes.
- Link to the spec ([docs/spec.md](../spec.md)) and the contract
  ([docs/dictionary-contract.md](../dictionary-contract.md)) for facts; don't copy them in.
- Every prompt follows [rules.md](rules.md).
