# bada ime

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/layouts/qwerty-bada-kana-dark.png">
  <img src="docs/images/layouts/qwerty-bada-kana-light.png" alt="QWERTY, bada and kana on one HHKB JIS keyboard">
</picture>

바다 입력기 · バダIME — a macOS input method for JIS keyboards with two modes: **muhenkan**
(QWERTY passthrough) and **henkan** (every enabled Korean, Japanese and Latin layout read at once,
ranked as you type).

Status: **0.1.2**, layouts 0, 10, 11, 12, 13 (bada), 14 (3-set JIS), 18 (Korean Roman), 19 (Korean Kana), 20 (kana),
21 (romaji), 150 (Symbols) and 151 (Unicode); henkan profiles for かな, 変換 and カタカナ. Build: `make` in `mac/` (needs Rust and the Command Line
Tools), `make install` to install. See [docs/spec.md](docs/spec.md) and [docs/bada-layout.md](docs/bada-layout.md).

## Layouts

**bada** (layout 13) types like 2-set, keeps finals like 3-set, and uses the whole key space like JIS
kana: digits and punctuation are on ⇧, so Korean typing never needs a prediction or a mode switch for them.
How it compares: [docs/layouts/13-bada.md](docs/layouts/13-bada.md).

Henkan mode has no sub-modes: every enabled layout reads the keys at once, and the readings are
ranked as you type. The keyboard above is **QWERTY + bada + kana** (layouts 0, 13, 20): English,
Korean and Japanese from one mode. Any other set works the same way, e.g. standard 2-set Korean
with English, **QWERTY + KTS** (0, 10):

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/layouts/qwerty-kts-dark.png">
  <img src="docs/images/layouts/qwerty-kts-light.png" alt="QWERTY and 2-set Korean on one HHKB JIS keyboard">
</picture>

Every layout, with its rules: [docs/layouts](docs/layouts/README.md). More combinations
(bada + romaji, QWERTY + romaji): [docs/layouts/combinations.md](docs/layouts/combinations.md).

MIT © Mirae Research

## Set up with an AI assistant

In a clone, tell a coding assistant: `Read docs/interactive-prompts/start.md and follow it.` It
installs bada ime and then builds your dictionary with you
([docs/dictionary-contract.md](docs/dictionary-contract.md)): practically required if you type 초성.

An example setup, an HHKB JIS with three henkan profiles on its thumb keys:
[docs/examples/hhkb-profiles](docs/examples/hhkb-profiles/README.md).

To learn your words from your own typing, type in [utils/learner-app](utils/learner-app/README.md)
(a local page logging a plain text field) and have an assistant merge its export into your dictionary.
