// Learn Korean (learn.html): lessons and drills on the engine. Each drill item is a word: its keys
// go to the engine as on the simulator, Space commits, and the commit is checked against the word.
// Lessons are in learn-content.js; progress is kept in local storage.

import { loadEngine } from "./load.js";
import { loadWeb } from "./settings.js";
import { LEVELS, LESSONS, ANTHEM, TWO_SET, lessonItems } from "./learn-content.js";
import { keyOf, isDrillKey } from "./learn-keys.js";

const $ = id => document.getElementById(id);
const STORE = "bada-learn";
const PASS = 0.9;

// ── Theme: the simulator's setting ─────────────────────────────────────────────────────────────
const dark = matchMedia("(prefers-color-scheme: dark)");
function applyTheme() {
  const theme = loadWeb().theme || "auto";
  document.documentElement.dataset.theme = theme === "auto" ? (dark.matches ? "dark" : "light") : theme;
}
dark.addEventListener("change", applyTheme);
applyTheme();

// ── Progress ───────────────────────────────────────────────────────────────────────────────────
function loadProgress() {
  try {
    const p = JSON.parse(localStorage.getItem(STORE) || "{}");
    return { lesson: p.lesson || LESSONS[0].id, onboarded: !!p.onboarded, best: p.best || {}, show: { std: true, keys: true, ...p.show } };
  } catch { return { lesson: LESSONS[0].id, onboarded: false, best: {}, show: { std: true, keys: true } }; }
}
const progress = loadProgress();
const save = () => { try { localStorage.setItem(STORE, JSON.stringify(progress)); } catch {} };

// ── State ──────────────────────────────────────────────────────────────────────────────────────
let engine = null;
let lesson = LESSONS.find(l => l.id === progress.lesson) || LESSONS[0];
const drill = { items: [], at: 0, results: [], pressed: 0, out: null, started: 0, shiftLatch: false };

// ── The lesson list ────────────────────────────────────────────────────────────────────────────
function drawLessons() {
  const nav = $("lessons");
  nav.replaceChildren(...LEVELS.map(level => {
    const box = document.createElement("div");
    box.className = "level";
    const h = document.createElement("h2");
    h.textContent = `Level ${level.id}: ${level.title}`;
    box.append(h);
    for (const l of level.lessons) {
      const b = document.createElement("button");
      b.className = "lesson-link" + (l.id === lesson.id ? " current" : "");
      if (l.id === lesson.id) b.setAttribute("aria-current", "page");
      const best = progress.best[l.id];
      b.innerHTML = `<span>${l.id} ${esc(l.title)}</span>` +
        (best != null ? `<span class="badge${best >= PASS ? " pass" : ""}" title="Best accuracy">${Math.round(best * 100)}%</span>` : "");
      b.onclick = () => openLesson(l.id);
      box.append(b);
    }
    return box;
  }));
}

function openLesson(id) {
  lesson = LESSONS.find(l => l.id === id) || LESSONS[0];
  progress.lesson = lesson.id;
  save();
  drawLessons();
  $("levelTitle").textContent = `Level ${lesson.level.id}: ${lesson.level.title}`;
  $("levelBody").innerHTML = lesson.level.intro;
  $("levelIntro").open = lesson.level.lessons[0].id === lesson.id;
  $("lessonTitle").textContent = `${lesson.id} ${lesson.title}`;
  $("lessonBody").innerHTML = lesson.anthem ? anthemBody() : lesson.body;
  $("drillTitle").textContent = lesson.anthem ? ANTHEM.title : "Drill";
  drawKeyboard();
  startDrill();
  window.scrollTo(0, 0);
}

function anthemBody() {
  const how = { roman: "every jamo spelled, with <code>;</code> where the blocks split another way",
    short: "with the shortcuts of this level, and without <code>;</code> where the word comes first anyway",
    twoset: "in 2-set keys" }[lesson.anthem];
  const keys = lessonItems(lesson).map(i => i.keys.length).reduce((a, b) => a + b, 0);
  const roman = lesson.anthem === "short"
    ? ` That's ${keys} keys, against ${ANTHEM.lines.flatMap(l => l.words).reduce((n, w) => n + w.roman.length, 0)} in Level 1.` : "";
  return `<p>The national anthem of Korea, 애국가 (Aegukga): verse 1 and the refrain, typed ${how}.${roman}</p>`;
}

// ── The drill ──────────────────────────────────────────────────────────────────────────────────
function startDrill() {
  drill.items = lessonItems(lesson);
  drill.at = 0;
  drill.results = [];
  drill.pressed = 0;
  drill.out = null;
  drill.started = 0;
  drill.shiftLatch = false;
  if (engine) {
    engine.applySettings({ enabled: [lesson.level.layout] });
    engine.setMode("henkan");
    engine.flush();
  }
  $("result").hidden = true;
  $("shiftBtn").hidden = lesson.level.layout !== 10;
  drawWords();
  drawTyping();
}

function wordEl(item, idx) {
  const d = document.createElement("span");
  const r = drill.results[idx];
  d.className = "word" + (idx === drill.at && drill.at < drill.items.length ? " current" : "") + (r ? (r.ok ? " ok" : " bad") : "");
  d.innerHTML = `<span class="std">${esc(item.rr || " ")}</span><span class="keys">${esc(item.keys)}</span>` +
    `<span class="ko">${esc(item.ko)}</span>` + (r && !r.ok ? `<span class="got">${esc(r.got || "–")}</span>` : "");
  return d;
}

function drawWords() {
  const host = $("words");
  host.classList.toggle("no-std", !progress.show.std);
  host.classList.toggle("no-keys", !progress.show.keys);
  host.classList.toggle("anthem", !!lesson.anthem);
  if (lesson.anthem) {
    let idx = 0;
    const lines = ANTHEM.lines.map(line => {
      const p = document.createElement("div");
      p.className = "line";
      const ws = document.createElement("div");
      ws.className = "line-words";
      for (const _ of line.words) ws.append(wordEl(drill.items[idx], idx++));
      const en = document.createElement("div");
      en.className = "en";
      en.textContent = line.en;
      p.append(ws, en);
      return p;
    });
    const credit = document.createElement("p");
    credit.className = "credit muted";
    credit.textContent = ANTHEM.credit;
    host.replaceChildren(...lines, credit);
  } else {
    host.replaceChildren(...drill.items.map(wordEl));
  }
  $("drillCount").textContent = `${Math.min(drill.at + 1, drill.items.length)} of ${drill.items.length}`;
  // Keep the current word in view inside the list (never scroll the page itself)
  const cur = host.querySelector(".current");
  if (cur && host.scrollHeight > host.clientHeight) {
    const top = cur.offsetTop, bottom = top + cur.offsetHeight;
    if (top < host.scrollTop) host.scrollTop = top - 8;
    else if (bottom > host.scrollTop + host.clientHeight) host.scrollTop = bottom - host.clientHeight + 8;
  }
}

function drawTyping() {
  const out = drill.out;
  $("preedit").textContent = out?.preedit || "";
  const list = $("readings");
  const menu = out && out.menu.length > 1 ? out.menu : [];
  list.replaceChildren(...menu.map((m, i) => {
    const li = document.createElement("li");
    li.textContent = m;
    if (i === out.selected) li.className = "sel";
    return li;
  }));
  // The next key on the keyboard
  document.querySelectorAll("#board .key.next").forEach(k => k.classList.remove("next"));
  const item = drill.items[drill.at];
  const next = item?.keys[drill.pressed];
  $("board").classList.toggle("shift-next", !!next && next !== next.toLowerCase());
  if (next && progress.show.keys) document.querySelector(`#board .key[data-code="${keyOf(next).code}"]`)?.classList.add("next");
  $("shiftBtn").setAttribute("aria-pressed", String(drill.shiftLatch));
}

/** A key from the keyboard or the screen. Returns whether it was taken. */
function press(code, shift) {
  if (!engine || drill.at >= drill.items.length) return false;
  const staged = !!drill.out?.preedit;
  if (isDrillKey(code)) {
    if (!drill.started) drill.started = performance.now();
    drill.out = engine.key(code, { shift });
    drill.pressed++;
  } else if (code === "Backspace") {
    if (!staged) return true;
    drill.out = engine.key("Backspace");
    drill.pressed = Math.max(0, drill.pressed - 1);
  } else if (code === "Tab" || code === "ArrowDown" || code === "ArrowUp") {
    if (!staged) return false;   // Tab moves the focus as usual
    drill.out = engine.key(code, { shift });
  } else if (code === "Space" || code === "Enter" || code === "NumpadEnter") {
    if (!staged) return true;
    const out = engine.key("Space");
    commit(out.commit.trim());
    return true;
  } else {
    return false;
  }
  drawTyping();
  return true;
}

function commit(got) {
  const item = drill.items[drill.at];
  drill.results[drill.at] = { ok: got === item.ko, got };
  drill.at++;
  drill.pressed = 0;
  drill.out = null;
  drill.shiftLatch = false;
  engine.flush();
  drawWords();
  drawTyping();
  if (drill.at >= drill.items.length) finish();
}

function finish() {
  const ok = drill.results.filter(r => r.ok).length;
  const acc = ok / drill.items.length;
  const minutes = Math.max((performance.now() - drill.started) / 60000, 1 / 60);
  const chars = drill.items.reduce((n, i) => n + [...i.ko].length, 0);
  const best = progress.best[lesson.id];
  if (best == null || acc > best) progress.best[lesson.id] = acc;
  save();
  drawLessons();
  const at = LESSONS.findIndex(l => l.id === lesson.id);
  const next = LESSONS[at + 1];
  const box = $("result");
  box.hidden = false;
  box.innerHTML = `<b>${acc >= PASS ? "Well done!" : "Done."}</b> ${ok} of ${drill.items.length} right (${Math.round(acc * 100)}%),
    ${Math.round(chars / minutes)} Hangul blocks a minute.`;
  const again = document.createElement("button");
  again.textContent = "Again";
  again.onclick = startDrill;
  box.append(" ", again);
  if (next) {
    const go = document.createElement("button");
    go.className = "primary";
    go.textContent = `Next: ${next.id} ${next.title}`;
    go.onclick = () => openLesson(next.id);
    box.append(" ", go);
  }
}

// ── The keyboard: the letter keys and ; only ───────────────────────────────────────────────────
const ROWS = [["qwertyuiop", 0], ["asdfghjkl;", 0.25], ["zxcvbnm", 0.75]];
const SPECIAL = { 13: {}, 15: { q: "ㅊ", x: "ㅇ", ";": "split" }, 10: {} };

function drawKeyboard() {
  const NS = "http://www.w3.org/2000/svg";
  const U = 58, G = 3, PAD = 10;
  const el = (tag, attrs, text) => {
    const e = document.createElementNS(NS, tag);
    for (const k in attrs) e.setAttribute(k, attrs[k]);
    if (text != null) e.textContent = text;
    return e;
  };
  const width = 10.25 * U + PAD * 2, height = 3 * U + PAD * 2;
  const twoSet = lesson.level.layout === 10;
  const svg = el("svg", { viewBox: `0 0 ${width} ${height}`, role: "img",
    "aria-label": twoSet ? "The letter keys, with 2-set jamo" : "The letter keys and semicolon" });
  ROWS.forEach(([keys, offset], r) => {
    [...keys].forEach((ch, c) => {
      const x = PAD + (offset + c) * U, y = PAD + r * U;
      const code = keyOf(ch).code;
      const g = el("g", { class: "key", "data-code": code });
      g.append(el("rect", { x: x + G, y: y + G, width: U - 2 * G, height: U - 2 * G, rx: 7, class: "cap" }));
      const latin = ch === ";" ? ";" : ch.toUpperCase();
      if (twoSet) {
        const base = Object.keys(TWO_SET).find(j => TWO_SET[j] === ch);
        const shifted = Object.keys(TWO_SET).find(j => TWO_SET[j] === ch.toUpperCase() && ch !== ";");
        g.append(el("text", { x: x + 10, y: y + 20, class: "latin small" }, latin));
        if (base) g.append(el("text", { x: x + U / 2, y: y + U - 12, "text-anchor": "middle", class: "jamo" }, base));
        if (shifted) g.append(el("text", { x: x + U - 9, y: y + 20, "text-anchor": "end", class: "jamo small" }, shifted));
      } else {
        g.append(el("text", { x: x + U / 2, y: y + U / 2 + 8, "text-anchor": "middle", class: "latin" }, latin));
        const extra = SPECIAL[lesson.level.layout]?.[ch];
        if (extra && (lesson.level.id >= 2 || ch === ";")) {
          g.append(el("text", { x: x + U - 9, y: y + U - 9, "text-anchor": "end", class: "jamo small" }, extra));
        }
      }
      g.addEventListener("pointerdown", e => {
        e.preventDefault();
        g.classList.add("on");
        const shift = drill.shiftLatch;
        drill.shiftLatch = false;
        press(code, shift);
      });
      g.addEventListener("pointerup", () => g.classList.remove("on"));
      g.addEventListener("pointerleave", () => g.classList.remove("on"));
      svg.append(g);
    });
  });
  $("board").replaceChildren(svg);
}

// ── Keys from the keyboard ─────────────────────────────────────────────────────────────────────
document.addEventListener("keydown", e => {
  if ($("onboarding").open || e.metaKey || e.ctrlKey || e.altKey || e.isComposing) return;
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLButtonElement && (e.code === "Space" || e.code === "Enter")) {
    if (e.target instanceof HTMLButtonElement) e.target.blur(); else return;
  }
  if (press(e.code, e.shiftKey)) {
    e.preventDefault();
    document.querySelector(`#board .key[data-code="${e.code}"]`)?.classList.add("on");
  }
});
document.addEventListener("keyup", e => document.querySelector(`#board .key[data-code="${e.code}"]`)?.classList.remove("on"));

// ── Controls ───────────────────────────────────────────────────────────────────────────────────
$("showStd").checked = progress.show.std;
$("showKeys").checked = progress.show.keys;
$("showStd").onchange = e => { progress.show.std = e.target.checked; save(); drawWords(); };
$("showKeys").onchange = e => { progress.show.keys = e.target.checked; save(); drawWords(); drawTyping(); };
$("restartBtn").onclick = () => startDrill();
$("shiftBtn").onclick = () => { drill.shiftLatch = !drill.shiftLatch; drawTyping(); };
$("tabBtn").onclick = () => press("Tab", false);
$("bsBtn").onclick = () => press("Backspace", false);
$("spaceBtn").onclick = () => press("Space", false);
for (const id of ["shiftBtn", "tabBtn", "bsBtn", "spaceBtn"]) $(id).addEventListener("pointerdown", e => e.preventDefault());

// ── Onboarding (optional) ──────────────────────────────────────────────────────────────────────
const onboarding = $("onboarding");
onboarding.addEventListener("close", () => {
  progress.onboarded = true;
  save();
  if (onboarding.returnValue === "start") openLesson(LESSONS[0].id);
});
$("howBtn").onclick = () => onboarding.showModal();

// ── Start ──────────────────────────────────────────────────────────────────────────────────────
function esc(s) {
  return String(s).replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
}

openLesson(lesson.id);
if (!progress.onboarded) onboarding.showModal();
loadEngine((got, total, phase) => {
  $("progress").style.width = `${Math.min(100, (100 * got) / total).toFixed(1)}%`;
  $("progressText").textContent = phase === "starting" ? "Starting the engine…" : `${(got / 1e6).toFixed(1)} of ${(total / 1e6).toFixed(1)} MB`;
}).then(loaded => {
  engine = loaded.engine;
  $("loading").hidden = true;
  startDrill();
}).catch(err => {
  $("progressText").textContent = `The engine didn't load: ${err.message}`;
});
