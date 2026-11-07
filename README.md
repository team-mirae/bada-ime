# bada ime

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/layouts/qwerty-bada-kana-dark.png">
  <img src="docs/images/layouts/qwerty-bada-kana-light.png" alt="QWERTY, bada and kana on one HHKB JIS keyboard">
</picture>

바다 입력기 · バダIME — a macOS input method for JIS keyboards with two modes: **muhenkan**
(QWERTY passthrough) and **henkan** (every enabled Korean, Japanese and Latin layout read at once,
ranked as you type).

Status: v0.1, layouts 0, 10, 11, 12, 13 (bada), 18 (Korean Roman), 19 (Korean Kana), 20 (kana) and 21 (romaji). Build: `make` in `mac/` (needs Rust and the Command Line
Tools), `make install` to install. See [docs/spec.md](docs/spec.md) and [docs/bada-layout.md](docs/bada-layout.md).

## Layouts

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
