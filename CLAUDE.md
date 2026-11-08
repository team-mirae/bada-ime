# bada ime

- Source of truth: `docs/spec.md` (the IME) and `docs/bada-layout.md` (layout 13). If code and docs
  disagree, the docs win or are updated in the same change. Don't implement **[decide]** items
  silently.
- Most code is ported from `~/Projects/jis-moonshot` and `~/Projects/jis-korean`; read their
  `docs/` before porting.
- Terms: muhenkan mode / henkan mode (無変換 / 変換; canonically 무변환 모드 / 변환 모드).
- Names: bada (lowercase), バダ (katakana), 바다.
- macOS only, JIS only. Caps Lock is out of scope.

## Building

- Engine: `cargo test` in `engine/`. macOS: `make` / `make install` in `mac/`.
- `engine/data/ko-words.tsv`: `python3 engine/tools/build_ko_words.py` (reads the jis-moonshot corpora).
  Then `python3 engine/tools/build_jamo_lm.py` (the typo model, `ko-jamo-4gram.bin`, from ko-words.tsv).
- `engine/data/emoji.tsv`, `unicode-names.tsv`: `python3 engine/tools/build_unicode_data.py` (Unicode files in
  `engine/tools/cache/unicode/`, URLs in the script). `symbols.tsv` is hand-written.
- `utils/learner-app`: `npm start` / `npm test` (Node 20+, no packages).
- Analysis tools need `tools/analysis/.venv` with numpy (see docs/bada-history.md §9).
