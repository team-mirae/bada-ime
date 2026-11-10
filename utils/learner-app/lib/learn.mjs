// Turns logged sessions into training data (`bada-learner-export/1`) and into a bada dictionary
// (`bada-dictionary` v1, docs/dictionary-contract.md).
//
// A session file is JSON lines. The first is `{"type":"meta",…}`; then events, `t` in ms since the
// session started:
//   {"type":"k","t","code","shift","alt","ctrl","meta","comp","repeat"}   keydown (comp: isComposing)
//   {"type":"u","t","code"}                                                 keyup
//   {"type":"cs","t"} / {"type":"cu","t","data"} / {"type":"ce","t","data"} composition start/update/end
//   {"type":"i","t","it","data"}                                            input (inputType, data)
//   {"type":"page","t","page","text"}                                       the page's text (latest wins)

const LIMITS = { positive: 20000 };
// Keys that move through the menu or end a composition: not part of what was read.
const CONTROL = new Set(["Tab", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Lang1", "Lang2", "Space", "Enter",
  "NumpadEnter", "Escape", "ShiftLeft", "ShiftRight", "AltLeft", "AltRight", "MetaLeft", "MetaRight", "ControlLeft",
  "ControlRight", "CapsLock", "Fn"]);
const NAV = new Set(["Tab", "ArrowUp", "ArrowDown", "Lang1"]);

export function parseSession(text) {
  const lines = text.split("\n").filter(Boolean).map(l => { try { return JSON.parse(l); } catch { return null; } }).filter(Boolean);
  const meta = lines.find(l => l.type === "meta") || { type: "meta", layouts: [] };
  return { meta, events: lines.filter(l => l.type !== "meta") };
}

/** Compositions committed with non-empty text, with the keys pressed for each. bada commits with
 * the terminating space or punctuation inside the composition ("그때 "): that tail is not the word.
 * bada takes menu keys (Tab, ↓, かな) before the browser sees them, so a menu move shows up as a
 * composition update with no keydown before it: `nav` counts those. */
export function commits(events) {
  const out = [];
  let keys = [], composing = false, startAt = 0, nav = 0, keyed = false;
  for (const e of events) {
    if (e.type === "k") { keys.push(e); keyed = true; }
    else if (e.type === "cs") {
      // The keydown that started it comes just before compositionstart.
      startAt = Math.max(0, keys.length - 1);
      composing = true; nav = 0; keyed = true;
    } else if (e.type === "cu") {
      if (!keyed) nav++;
      keyed = false;
    } else if (e.type === "ce") {
      const pressed = composing ? keys.slice(startAt) : [];
      composing = false;
      keys = [];
      const text = (e.data || "").replace(/^\s+/, "").replace(/[\s\p{P}\p{S}]+$/u, "");
      if (!text) continue;
      out.push({
        text,
        t: e.t,
        keys: pressed.filter(k => !CONTROL.has(k.code) && k.code !== "Backspace")
          .map(k => (k.shift ? "⇧" : "") + k.code),
        backspaces: pressed.filter(k => k.code === "Backspace").length,
        nav: nav + pressed.filter(k => NAV.has(k.code)).length,
        alt: pressed.some(k => k.alt && !CONTROL.has(k.code)),
      });
    } else if (e.type === "i" && !composing && e.it !== "insertCompositionText") keys = [];
  }
  return out;
}

const KOREAN = /[가-힣]+/g, ENGLISH = /[A-Za-z][A-Za-z'’-]*[A-Za-z]|[A-Za-z]/g, JAPANESE = /[ぁ-ヿ㐀-鿿々ー]+/g;
function countWords(text, into) {
  const add = (lang, w) => into[lang].set(w, (into[lang].get(w) || 0) + 1);
  for (const w of text.match(KOREAN) || []) add("korean", w);
  for (const w of text.match(ENGLISH) || []) if (w.length > 1) add("english", w.toLowerCase());
  for (const w of text.match(JAPANESE) || []) add("japanese", w);
}
const sorted = m => [...m].map(([word, count]) => ({ word, count })).sort((a, b) => b.count - a.count || (a.word < b.word ? -1 : 1));

/** Everything learned from these sessions (`bada-learner-export/1`). */
export function learn(sessions, { texts = false } = {}) {
  const words = { korean: new Map(), english: new Map(), japanese: new Map() };
  const keyCounts = new Map(), rejected = new Map(), pagesOut = [];
  const usedLayouts = new Set();
  let keystrokes = 0, nCommits = 0, nPages = 0;
  for (const { meta, events } of sessions) {
    (meta.layouts || []).forEach(id => usedLayouts.add(id));
    const pages = new Map();
    for (const e of events) {
      if (e.type === "page") pages.set(e.page, e.text || "");
      if (e.type === "k" && !e.repeat) {
        keystrokes++;
        const k = (e.shift ? "⇧" : "") + (e.alt ? "⌥" : "") + e.code;
        keyCounts.set(k, (keyCounts.get(k) || 0) + 1);
      }
    }
    nPages += pages.size;
    const finalText = [...pages.values()].join("\n");
    countWords(finalText, words);
    if (texts) pagesOut.push(...[...pages.values()].filter(Boolean));
    for (const c of commits(events)) {
      nCommits++;
      // Committed, then gone from the page: a hint that it was the wrong reading.
      if (!finalText.includes(c.text)) {
        const row = rejected.get(c.text) || { word: c.text, count: 0 };
        row.count++;
        rejected.set(c.text, row);
      }
    }
  }
  const out = {
    format: "bada-learner-export",
    version: 1,
    created: new Date().toISOString(),
    sessions: sessions.length,
    layouts: [...usedLayouts].sort((a, b) => a - b),
    totals: { keystrokes, commits: nCommits, pages: nPages },
    words: { korean: sorted(words.korean), english: sorted(words.english), japanese: sorted(words.japanese) },
    rejected: [...rejected.values()].sort((a, b) => b.count - a.count),
    keys: Object.fromEntries([...keyCounts].sort((a, b) => b[1] - a[1])),
  };
  if (texts) out.texts = pagesOut;
  return out;
}

/** A `bada-dictionary` v1 file from an export: words used at least `min` times. */
export function toDictionary(exp, { min = 2 } = {}) {
  const positive = [];
  for (const language of ["korean", "english", "japanese"])
    for (const { word, count } of exp.words[language]) if (count >= min) positive.push({ word, language, count });
  positive.sort((a, b) => b.count - a.count);
  return {
    format: "bada-dictionary",
    version: 1,
    source: { tool: "bada learner-app", created: exp.created.slice(0, 10), description: `${exp.sessions} session(s), ${exp.totals.keystrokes} keystrokes; words used at least ${min}×` },
    positive: positive.slice(0, LIMITS.positive).map(p => p.language === "korean" ? { word: p.word, count: p.count } : p),
    negative: [],
  };
}
