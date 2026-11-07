# bada ime

바다 입력기 · バダIME — a macOS input method for JIS keyboards with two modes: **muhenkan**
(QWERTY passthrough) and **henkan** (every enabled Korean, Japanese and Latin layout read at once,
ranked as you type).

Status: v0.1, layouts 0, 10, 13 (bada), 18 (Korean Roman), 19 (Korean Kana), 20 (kana) and 21 (romaji). Build: `make` in `mac/` (needs Rust and the Command Line
Tools), `make install` to install. See [docs/spec.md](docs/spec.md) and [docs/bada-layout.md](docs/bada-layout.md).

MIT © Mirae Research

## Set up with an AI assistant

In a clone, tell a coding assistant: `Read docs/interactive-prompts/start.md and follow it.` It
installs bada ime and then builds your dictionary with you
([docs/dictionary-contract.md](docs/dictionary-contract.md)): practically required if you type 초성.
