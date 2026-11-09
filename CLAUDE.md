# bada ime

- Source of truth: `docs/spec.md` (the IME) and `docs/bada-layout.md` (layout 13). If code and docs
  disagree, the docs win or are updated in the same change. Don't implement **[decide]** items
  silently.
- Most code is ported from `~/Projects/jis-moonshot` and `~/Projects/jis-korean`; read their
  `docs/` before porting.
- Terms: muhenkan mode / henkan mode (無変換 / 変換; canonically 무변환 모드 / 변환 모드).
- Names: bada (lowercase), バダ (katakana), 바다.
- A new layout page `docs/layouts/NNN-name.md` needs the metadata header and follows the maintainer
  rules in `docs/layouts/README.md`; its in-code name must be unique.
- macOS only, JIS only. Caps Lock is out of scope.

## Building

- Engine: `cargo test` in `engine/`. macOS: `make` / `make install` in `mac/`.
- `engine/data/ko-words.tsv`: `python3 engine/tools/build_ko_words.py` (reads the jis-moonshot corpora).
  Then `python3 engine/tools/build_jamo_lm.py` (the typo model, `ko-jamo-4gram.bin`, from ko-words.tsv).
- `engine/data/emoji.tsv`, `unicode-names.tsv`: `python3 engine/tools/build_unicode_data.py` (Unicode files in
  `engine/tools/cache/unicode/`, URLs in the script). `symbols.tsv` and `unicode-aliases.tsv` are hand-written.
- Typing robot (`docs/robot.md`): `cargo test --release --test robot`. Corpora: `engine/tests/data/robot/`,
  from `tools/analysis/.venv/bin/python engine/tools/build_robot_corpus.py` (sources in `engine/tools/cache/robot/`).
  Layout JSONs for the layouts defined in code: `cargo run --example layout_json`.
  Each JSON claims a reliability per language (perfect / production / experimental); CI
  (`.github/workflows/robot.yml`) fails a layout that doesn't earn its claim.
- `engine/data/ja-kana-native.txt` and `ja-common-words.txt`: `tools/analysis/.venv/bin/python engine/tools/build_kana_native.py`.
- Web simulator (`web/`, bada.sjun.me): `node web/build.mjs` → `web/dist`; `node web/test/smoke.mjs`. Deployed to Vercel
  by `.github/workflows/deploy-web.yml` on push to main. It ports the Mac shell's behaviour (InputController,
  ProfileKeys, Settings, SettingsWindow): a change there needs the same change in `web/src`.
- `utils/learner-app`: `npm start` / `npm test` (Node 20+, no packages).
- Analysis tools need `tools/analysis/.venv` with numpy (see docs/bada-history.md §9).
