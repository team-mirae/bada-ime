# bada ime · web

[bada.sjun.me](https://bada.sjun.me): bada ime in a browser. The engine (`engine/`) compiled to
WebAssembly runs behind a text field, with the macOS shell's behaviour ported to the page:

- **Keys** as `mac/Sources/InputController.swift` and `LanguageKeyTap.swift` handle them: 英数 and
  かな, HYPER / MEH + かな, henkan profiles and their recorded keys (`ProfileKeys.swift`), marked
  text, the suggestion menu, the selection for hanja and 半角/全角. Keys the engine passes on type
  layout 0's JIS legends, as the ABC layout does on the Mac.
- **Settings**: every page of the Mac's settings window, kept in the browser's local storage, in the
  same JSON. A `bada-settings/1` file exported from the Mac imports here and back. The user
  dictionary is kept in local storage too.
- **The keyboard** on screen: HHKB JIS by default, Apple-style JIS as an option (Settings →
  Keyboard), drawn as `tools/render` draws it, with the legends of the layouts in use (from
  `docs/layouts/*.json`). Keys can be clicked; ⇧ ⌃ ⌥ ⌘ latch for the next key.
- Without 英数 and かな keys: tap left ⌘ alone for 英数 and right ⌘ alone for かな (Settings →
  Keyboard), or click them on screen.

The page needs the OS input source on a plain keyboard (ABC): an OS input method that composes
keeps the keys from it.

## Build

```sh
node web/build.mjs                  # web/dist (needs Rust with the wasm32-unknown-unknown target)
python3 -m http.server -d web/dist  # then open http://localhost:8000
node web/test/smoke.mjs             # the engine as WebAssembly, typing on the shipped data
```

`web/wasm` is a small crate over the engine with a raw WebAssembly ABI (no wasm-bindgen): the ops
of `engine/include/bada.h`, JSON in and out (`web/src/engine.js`). The data files are gzipped and
named by hash (about 16 MB to download, cached for good); the page unzips them with
`DecompressionStream`.

`node web/build.mjs --vercel` also writes `web/.vercel/output` for `vercel deploy --prebuilt`, which
`.github/workflows/deploy-web.yml` runs on every push to main that touches the page, the engine or
its data (secret `VERCEL_ACCESS_TOKEN`).
