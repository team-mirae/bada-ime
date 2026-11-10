import { test } from "node:test";
import assert from "node:assert/strict";
import { commits, learn, parseSession, toDictionary } from "../lib/learn.mjs";
import { loadLayouts } from "../lib/layouts.mjs";

const layouts = loadLayouts();
const bada = layouts.find(l => l.id === 13);
// A key in layout 13 that types this consonant unshifted.
const keyFor = jamo => Object.entries(bada.keys).find(([, k]) => k.base && ["c", "i"].includes(k.base.c) && k.base.t.split("/").includes(jamo))[0];

// Keys typed with bada ime in a browser: the first keydown comes before compositionstart.
function typed(codes, text, extra = []) {
  const ev = [];
  codes.forEach((code, i) => {
    ev.push({ type: "k", code, shift: false, alt: false, comp: i > 0 });
    if (i === 0) ev.push({ type: "cs" });
    ev.push({ type: "cu", data: "x" });
  });
  for (const code of extra) ev.push({ type: "k", code, shift: false, alt: false, comp: true });
  ev.push({ type: "ce", data: text }, { type: "i", it: "insertCompositionText", data: text });
  return ev;
}

function session(events, text) {
  const lines = [{ type: "meta", layouts: [0, 13] }, ...events, { type: "page", page: 1, text }];
  return parseSession(lines.map(l => JSON.stringify(l)).join("\n"));
}

test("words used often become positive words, a deleted commit a rejection", () => {
  const keys = [keyFor("ㅂ"), keyFor("ㄷ")];
  const events = [
    ...typed(keys, "바다", ["Space"]),
    { type: "i", it: "insertText", data: " " },
    ...typed(keys, "바다", ["Space"]),
    ...typed(keys, "보도", ["Space"]),
    { type: "k", code: "Backspace", shift: false, alt: false, comp: false },
    { type: "i", it: "deleteContentBackward", data: null },
  ];
  const s = session(events, "바다 바다 키로깅 hello Hello");
  assert.equal(commits(s.events).length, 3);
  assert.deepEqual(commits(s.events)[0].keys, keys);
  const exp = learn([s]);
  assert.deepEqual(exp.rejected, [{ word: "보도", count: 1 }]);
  assert.deepEqual(exp.words.english, [{ word: "hello", count: 2 }]);
  const dict = toDictionary(exp, { min: 2 });
  assert.equal(dict.format, "bada-dictionary");
  assert.deepEqual(dict.positive.map(p => p.word).sort(), ["hello", "바다"]);
  assert.ok(!dict.positive.some(p => p.word === "키로깅"), "used once: below min");
});

test("bada's commits: the space inside the composition, menu moves without keys", () => {
  const [g, dd] = [keyFor("ㄱ"), keyFor("ㄸ")];
  const events = [
    { type: "k", code: g, shift: false, alt: false, comp: false }, { type: "cs" }, { type: "cu", data: "ㄱ" },
    { type: "k", code: dd, shift: false, alt: false, comp: true }, { type: "cu", data: "그동안" },
    { type: "cu", data: "그때" },   // ↓ in bada's menu: no keydown reaches the page
    { type: "k", code: "Space", key: " ", shift: false, alt: false, comp: true }, { type: "cu", data: "그때 " },
    { type: "ce", data: "그때 " },
  ];
  const s = session(events, "아마 그때 ");
  const [c] = commits(s.events);
  assert.equal(c.text, "그때");
  assert.equal(c.nav, 1);
  assert.equal(learn([s]).rejected.length, 0);
});
