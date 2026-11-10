"use strict";
// bada learner: logs a plain textarea typed with the system input method. Event format: lib/learn.mjs.
const $ = id => document.getElementById(id);
const NS = "http://www.w3.org/2000/svg";
const el = (tag, attrs, text) => { const e = document.createElementNS(NS, tag); for (const k in attrs) e.setAttribute(k, attrs[k]); if (text != null) e.textContent = text; return e; };
const U = 60, PAD = 14, G = 3, R = 7;
const LAYOUTS = BADA_LAYOUTS.filter(l => l.id < 300).sort((a, b) => a.id - b.id);

const state = { kb: "hhkb", layouts: [0, 13, 20] };
try { Object.assign(state, JSON.parse(localStorage.getItem("bada-learner") || "{}")); } catch (e) {}
const save = () => { try { localStorage.setItem("bada-learner", JSON.stringify(state)); } catch (e) {} };

// ---------- boards ----------
function rect(x, y, w, h) {
  const r = Math.min(R, w / 2, h / 2);
  return `M${x + r},${y}H${x + w - r}Q${x + w},${y} ${x + w},${y + r}V${y + h - r}Q${x + w},${y + h} ${x + w - r},${y + h}H${x + r}Q${x},${y + h} ${x},${y + h - r}V${y + r}Q${x},${y} ${x + r},${y}Z`;
}
function enter(x, y) {
  const x0 = x + G, x1 = x + 0.25 * U + G, x2 = x + 1.5 * U - G, y0 = y + G, ym = y + U - G, y2 = y + 2 * U - G;
  return `M${x0},${y0}H${x2}V${y2}H${x1}V${ym}H${x0}Z`;
}
let byCode = {};   // code -> [g, …] (one per board)
function drawBoards() {
  const kb = BADA_KEYBOARDS[state.kb] || BADA_KEYBOARDS.hhkb;
  byCode = {};
  const boards = state.layouts.map(id => LAYOUTS.find(l => l.id === id)).filter(Boolean).map(lay => {
    const W = kb.width * U + PAD * 2, H = kb.height * U + PAD * 2 + 22;
    const svg = el("svg", { viewBox: `0 0 ${W} ${H}`, role: "img", "aria-label": `${lay.name} on ${kb.name}` });
    svg.append(el("rect", { x: 0.5, y: 0.5, width: W - 1, height: H - 1, rx: 14, fill: "var(--case)", stroke: "var(--case-edge)" }));
    svg.append(el("text", { x: PAD + 2, y: 20, "font-size": 12, class: "modl" }, `${lay.id} · ${lay.name}`));
    for (const [code, kx, ky, kw, kh, label, shape] of kb.keys) {
      const x = PAD + kx * U, y = PAD + 22 + ky * U, w = kw * U, h = kh * U;
      const entry = code && lay.keys[code];
      const home = code === "KeyF" || code === "KeyJ";   // the homing keys
      const g = el("g", { class: "key" + (entry ? "" : " mod") + (home ? " home" : "") });
      g.append(el("path", { d: shape === "ENTER" ? enter(x, y) : rect(x + G, y + G, w - 2 * G, h - 2 * G), class: "cap" }));
      if (home) g.append(el("rect", { x: x + w / 2 - 9, y: y + h - 9, width: 18, height: 3, rx: 1.5, class: "homebar" }));
      if (entry) {
        const b = entry.base, s = entry.shift;
        if (b) g.append(el("text", { x: x + 11, y: y + h - 13, "font-size": b.t.split("/")[0].length > 2 ? 14 : 22, class: `lg c-${b.r ? b.r[0] : b.c}` }, b.t.split("/")[0]));
        if (b && b.r) g.append(el("text", { x: x + w - 9, y: y + h / 2 + 4, "font-size": 14, "text-anchor": "end", class: "lg c-f", "text-decoration": "underline" }, b.t.split("/")[1]));
        if (s && (!b || s.t !== b.t)) g.append(el("text", { x: x + 11, y: y + 23, "font-size": 13, class: `lg c-${s.c}`, opacity: .8 }, s.t.split("/")[0]));
        const jis = BADA_JIS[code];
        if (jis) g.append(el("text", { x: x + w - 9, y: y + h - 10, "font-size": 10, "text-anchor": "end", class: "jis" }, code.startsWith("Key") ? jis[1] : jis[0]));
      } else if (label != null) {
        const lines = label.split("/");
        lines.forEach((ln, i) => g.append(el("text", { x: x + w / 2, y: y + h / 2 + 4 + (i - (lines.length - 1) / 2) * 13, "font-size": h < U ? 11 : 12, "text-anchor": "middle", class: "modl" }, ln)));
      }
      svg.append(g);
      if (code) (byCode[code] ||= []).push(g);
    }
    const div = document.createElement("div"); div.className = "board"; div.append(svg);
    return div;
  });
  $("boards").replaceChildren(...boards);
}
for (const [id, k] of Object.entries(BADA_KEYBOARDS)) $("kb").append(new Option(k.name, id));
$("kb").value = state.kb;
$("kb").onchange = e => { state.kb = e.target.value; save(); drawBoards(); };
for (const l of LAYOUTS) {
  const lab = document.createElement("label"), box = document.createElement("input");
  box.type = "checkbox"; box.checked = state.layouts.includes(l.id);
  box.onchange = () => {
    state.layouts = LAYOUTS.filter(x => x.id === l.id ? box.checked : state.layouts.includes(x.id)).map(x => x.id);
    save(); drawBoards();
    if (session && sent + queue.length > 1) startSession();   // a session records the layouts it was typed with
  };
  lab.append(box, ` ${l.id} ${l.display || l.name}`);
  $("layouts").append(lab);
}
drawBoards();

// ---------- logging ----------
const t0 = () => Math.round(performance.now() - sessionStart);
let session = null, sessionStart = 0, page = 1, queue = [], sent = 0, keys = 0, nCommits = 0;
const id = () => new Date().toISOString().replace(/[:.]/g, "-") + "-" + Math.random().toString(36).slice(2, 6).padEnd(4, "0");
function startSession() {
  flush();
  session = id(); sessionStart = performance.now(); page = 1; sent = 0; keys = 0; nCommits = 0;
  queue.push({ type: "meta", format: "bada-learner-session/1", started: new Date().toISOString(), layouts: [...state.layouts],
    keyboard: state.kb, userAgent: navigator.userAgent });
  $("log").replaceChildren();
  showSession();
}
function log(e) {
  if (!session) startSession();
  queue.push({ ...e, t: t0() });
}
async function flush(keepalive = false) {
  if (!session || !queue.length) return;
  const events = queue; queue = [];
  try {
    const r = await fetch("/api/events", { method: "POST", headers: { "content-type": "application/json" }, keepalive,
      body: JSON.stringify({ session, events }) });
    if (!r.ok) throw new Error((await r.json()).error);
    sent += events.length;
    $("status").textContent = `Logging session ${session} · ${sent} events saved`;
  } catch (err) {
    queue = events.concat(queue);
    $("status").textContent = `Not saved (${err.message}); retrying. Is the server running?`;
  }
}
setInterval(flush, 1500);
addEventListener("pagehide", () => { snapshot(); flush(true); });

// The page's text, so words are counted from what was kept, not what was deleted.
let snapTimer = null, lastSnap = "";
function snapshot() {
  clearTimeout(snapTimer); snapTimer = null;
  const text = $("pad").value;
  if (text === lastSnap) return;
  lastSnap = text;
  log({ type: "page", page, text });
}
const scheduleSnapshot = () => { if (!snapTimer) snapTimer = setTimeout(snapshot, 3000); };

const pad = $("pad");
let composing = false, compKeys = [];
pad.addEventListener("keydown", e => {
  log({ type: "k", code: e.code, key: e.key, shift: e.shiftKey, alt: e.altKey, ctrl: e.ctrlKey, meta: e.metaKey, comp: e.isComposing, repeat: e.repeat });
  if (!e.repeat) keys++;
  compKeys.push((e.shiftKey ? "⇧" : "") + e.code.replace(/^Key|^Digit/, ""));
  (byCode[e.code] || []).forEach(g => g.classList.add("on"));
  if (e.code === "Tab" && !e.isComposing) e.preventDefault();   // keep focus here
  showSession();
});
pad.addEventListener("keyup", e => {
  log({ type: "u", code: e.code });
  (byCode[e.code] || []).forEach(g => g.classList.remove("on"));
});
addEventListener("blur", () => Object.values(byCode).flat().forEach(g => g.classList.remove("on")));
pad.addEventListener("compositionstart", () => { log({ type: "cs" }); composing = true; compKeys = compKeys.slice(-1); });
pad.addEventListener("compositionupdate", e => log({ type: "cu", data: e.data }));
pad.addEventListener("compositionend", e => {
  log({ type: "ce", data: e.data });
  composing = false;
  if (e.data) {
    nCommits++;
    const row = document.createElement("div");
    row.append(compKeys.join(" ") + " → ", Object.assign(document.createElement("b"), { textContent: e.data }));
    $("log").prepend(row);
    while ($("log").childElementCount > 50) $("log").lastChild.remove();
  }
  compKeys = [];
  showSession();
});
pad.addEventListener("input", e => {
  log({ type: "i", it: e.inputType, data: e.data });
  if (!composing && e.inputType !== "insertCompositionText") compKeys = [];
  scheduleSnapshot();
});
function showSession() {
  $("sKeys").textContent = keys; $("sCommits").textContent = nCommits; $("sPage").textContent = page;
}

$("newPage").onclick = () => { snapshot(); flush(); page++; pad.value = ""; lastSnap = ""; showSession(); pad.focus(); };
$("newSession").onclick = () => { snapshot(); startSession(); pad.value = ""; lastSnap = ""; pad.focus(); };

// ---------- stats ----------
const chips = (rows, f) => rows.length ? rows.map(f).join("") : `<span class="note">none yet</span>`;
const esc = s => String(s).replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
async function refreshStats() {
  try {
    const s = await (await fetch("/api/stats")).json();
    $("allStats").innerHTML = `<span>sessions <b>${s.sessions}</b></span><span>keys <b>${s.totals.keystrokes}</b></span>` +
      `<span>commits <b>${s.totals.commits}</b></span><span>Korean words <b>${s.words.korean}</b></span>` +
      `<span>English <b>${s.words.english}</b></span><span>Japanese <b>${s.words.japanese}</b></span>`;
    $("topWords").innerHTML = chips(s.top, w => `<span>${esc(w.word)}<b>${w.count}</b></span>`);
    $("rejected").innerHTML = chips(s.rejected, r => `<span>${esc(r.word)}<b>${r.count}</b></span>`);
  } catch (e) {}
}
refreshStats();
setInterval(refreshStats, 10000);

// ---------- export ----------
const query = () => `?min=${$("min").value}${$("texts").checked ? "&texts=1" : ""}`;
async function download(kind) {
  snapshot(); await flush();
  const r = await fetch(`/api/export/${kind}${query()}`);
  if (!r.ok) { $("exportStatus").textContent = `Export failed: ${(await r.json()).error}`; return; }
  const blob = await r.blob(), a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = (r.headers.get("content-disposition").match(/filename="([^"]+)"/) || [])[1] || `${kind}.json`;
  a.click();
  $("exportStatus").textContent = `Saved ${r.headers.get("x-saved-as")} (and downloaded).`;
  refreshStats();
}
$("exportTraining").onclick = () => download("training");
$("exportDict").onclick = () => download("dictionary");
$("install").onclick = () => $("confirm").classList.add("open");
$("installNo").onclick = () => $("confirm").classList.remove("open");
$("installYes").onclick = async () => {
  snapshot(); await flush();
  const r = await fetch("/api/install", { method: "POST", headers: { "content-type": "application/json" },
    body: JSON.stringify({ confirm: "replace", min: Number($("min").value) }) });
  const j = await r.json();
  $("confirm").classList.remove("open");
  $("exportStatus").textContent = r.ok
    ? `Wrote ${j.path}: ${j.positive} words.${j.backup ? ` Old file kept as ${j.backup}.` : ""} bada loads it the next time it's activated.`
    : `Not written: ${j.error}`;
};
pad.focus();
