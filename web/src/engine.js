// The engine (web/wasm, engine/ compiled to WebAssembly), as the macOS shell's Engine.swift has it:
// JSON in, JSON out. Works in the browser and in Node (web/test).

// Data files in the wasm crate's slot order (web/wasm/src/lib.rs).
export const DATA_FILES = [
  "ko-words.tsv", "en-words.txt", "hanja.tsv", "oecd.tsv", "ja-words.txt",
  "kanji.bin", "ko-jamo-4gram.bin", "symbols.tsv", "emoji.tsv", "unicode-names.tsv", "en-ko.tsv",
];

const OP = {
  key: 0, settings: 1, mode: 2, profile: 3, flush: 4, selection: 5, catalog: 6,
  dictionary: 7,
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export class Engine {
  /** `wasm`: the module's bytes or a compiled WebAssembly.Module. */
  static async load(wasm) {
    const { instance } = wasm instanceof WebAssembly.Module
      ? { instance: await WebAssembly.instantiate(wasm, {}) }
      : await WebAssembly.instantiate(wasm, {});
    return new Engine(instance.exports);
  }

  constructor(exports) {
    this.x = exports;
  }

  /** Hands data file `slot` (DATA_FILES order) to the engine. */
  put(slot, bytes) {
    const ptr = this.x.alloc(bytes.length);
    new Uint8Array(this.x.memory.buffer, ptr, bytes.length).set(bytes);
    this.x.data_put(slot, ptr, bytes.length);
  }

  /** Parses the data given and starts the engine. */
  init() {
    this.x.init();
  }

  call(op, arg = "") {
    const bytes = encoder.encode(arg);
    const ptr = this.x.alloc(bytes.length);
    new Uint8Array(this.x.memory.buffer, ptr, bytes.length).set(bytes);
    this.x.call(op, ptr, bytes.length);
    return decoder.decode(new Uint8Array(this.x.memory.buffer, this.x.reply_ptr(), this.x.reply_len()));
  }

  json(op, arg) {
    try { return JSON.parse(this.call(op, arg)) ?? {}; } catch { return {}; }
  }

  key(code, { shift = false, ctrl = false, alt = false, meta = false } = {}) {
    return output(this.json(OP.key, JSON.stringify({ key: code, shift, ctrl, alt, meta })));
  }
  setMode(mode) { return output(this.json(OP.mode, mode === "muhenkan" ? "0" : "1")); }
  setProfile(name) { return output(this.json(OP.profile, name)); }
  flush() { return output(this.json(OP.flush)); }
  hostSelection(text) { return output(this.json(OP.selection, text)); }
  /** Settings JSON (engine::Settings); returns the settings in effect. */
  applySettings(settings) { return this.json(OP.settings, JSON.stringify(settings)); }
  catalog() { return this.json(OP.catalog); }
  /** The user dictionary (docs/dictionary-contract.md); "" clears it. */
  loadDictionary(json) { return this.json(OP.dictionary, json); }
}

/** An Output with every field present (engine/src/engine.rs `Output`). */
function output(o) {
  return {
    commit: o.commit ?? "", replace_selection: !!o.replace_selection, preedit: o.preedit ?? "",
    menu: o.menu ?? [], selected: o.selected ?? 0, zero: o.zero ?? 0, consumed: !!o.consumed,
    horizontal: !!o.horizontal, notes: o.notes ?? [], japanese: o.japanese ?? [],
    need_selection: o.need_selection ?? null,
  };
}
