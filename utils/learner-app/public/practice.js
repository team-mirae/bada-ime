// Copy practice (/practice): one sentence of a filing at a time, typed with bada ime in a plain textarea.
// A mistake is counted when a wrong syllable is committed (composition finished), even if fixed later.
import { score, syllables } from "/practice-lib.js";

const $ = id => document.getElementById(id);
const box = $("box");
let corpus = null, queue = [], cur = null, done = new Set();
let started = 0, keys = 0, backspaces = 0, live = [], judged = 0, composing = false, compStart = 0;

const pct = x => `${(x * 100).toFixed(1)}%`;
const esc = s => s.replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);

async function load() {
  const res = await fetch("/api/corpus");
  if (!res.ok) {
    $("target").textContent = "No corpus yet. In utils/learner-app, run: node tools/fetch-dart.mjs";
    box.disabled = true;
    return;
  }
  corpus = await res.json();
  // DART writes the middle dot as ㆍ (U+318D); bada roman's ⇧\ types · (U+00B7). Same mark: show the one you can type.
  for (const c of corpus.companies) c.sentences = c.sentences.map(t => t.replace(/\u318D/g, "\u00B7"));
  const sel = $("company");
  sel.innerHTML = `<option value="">All companies (${corpus.companies.reduce((n, c) => n + c.sentences.length, 0)})</option>` +
    corpus.companies.map(c => `<option value="${esc(c.company)}">${esc(c.company)} (${c.sentences.length})</option>`).join("");
  try { const saved = JSON.parse(localStorage.getItem("bada-practice") || "{}"); if (saved.company != null) sel.value = saved.company; if (saved.order) $("order").value = saved.order; } catch {}
  await refreshStats();
  build();
  next();
}

function build() {
  const company = $("company").value, skip = $("skipDone").checked;
  queue = [];
  for (const c of corpus.companies) {
    if (company && c.company !== company) continue;
    c.sentences.forEach((text, index) => {
      if (!skip || !done.has(`${c.rcept_no}:${index}`)) queue.push({ company: c.company, rcept_no: c.rcept_no, url: c.url, index, total: c.sentences.length, text });
    });
  }
  if ($("order").value === "random") for (let i = queue.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); [queue[i], queue[j]] = [queue[j], queue[i]]; }
  try { localStorage.setItem("bada-practice", JSON.stringify({ company, order: $("order").value })); } catch {}
}

function next() {
  cur = queue.shift() || null;
  box.value = "";
  reset();
  if (!cur) {
    $("src").textContent = "";
    $("target").textContent = "Every sentence here is typed. Pick another company, or untick “Skip sentences already typed”.";
    return;
  }
  $("src").innerHTML = `<span>${esc(cur.company)}</span><span>${esc(corpus.report)}</span><span>Sentence ${cur.index + 1} of ${cur.total}</span>` +
    `<a href="${esc(cur.url)}" target="_blank" rel="noopener">Filing on DART</a>`;
  paint();
  box.focus();
}

function reset() {
  started = 0; keys = 0; backspaces = 0; live = []; judged = 0;
  $("nAcc").textContent = "–"; $("nCpm").textContent = "–"; $("nMade").textContent = "0"; $("nBs").textContent = "0";
}

/** The committed part of the box: without the syllable bada ime is still composing. */
function committed() {
  return composing ? box.value.slice(0, compStart) : box.value;
}

/**
 * Judges newly committed characters against the target. Only the first character that goes wrong
 * counts: after it, nothing more is judged until the text is right again up to there (a slip that
 * shifts everything after it is one mistake, not one per character).
 */
function judge() {
  const typed = [...committed()], target = [...cur.text];
  if (typed.length < judged) judged = typed.length;   // deleted: those places are judged again when retyped
  let ok = 0;
  while (ok < typed.length && typed[ok] === target[ok]) ok++;
  if (judged < ok) judged = ok;
  if (judged < typed.length && judged === ok) {
    live.push({ expected: target[judged] || "", got: typed[judged] });
    judged = typed.length;
  }
  $("nMade").textContent = live.length;
}

function paint() {
  const typed = [...committed()], target = [...cur.text];
  $("target").innerHTML = target.map((ch, i) => {
    const cls = i >= typed.length ? "p" : typed[i] === ch ? "ok" : "wrong";
    return `<span class="${cls}${i === typed.length ? " cur" : ""}">${esc(ch)}</span>`;
  }).join("");
  if (started && typed.length) {
    const min = (performance.now() - started) / 60000;
    $("nCpm").textContent = Math.round(syllables(typed.join("")) / Math.max(min, 1 / 60));
    $("nAcc").textContent = pct(score(cur.text.slice(0, typed.length), typed.join("")).accuracy);
  }
}

box.addEventListener("keydown", e => {
  if (!cur) return;
  if (!started) started = performance.now();
  if (e.key === "Escape") { box.value = ""; reset(); paint(); return; }
  if (e.key === "Enter" && !e.isComposing && !e.shiftKey) {
    e.preventDefault();
    submit();
    return;
  }
  if (!e.repeat) keys++;
  if (e.key === "Backspace") { backspaces++; $("nBs").textContent = backspaces; }
});
box.addEventListener("compositionstart", () => { composing = true; compStart = box.selectionStart; });
box.addEventListener("compositionend", () => { composing = false; judge(); paint(); });
box.addEventListener("input", e => {
  if (!cur) return;
  if (!e.isComposing && !composing) judge();
  paint();
});

async function submit() {
  if (!cur) return;
  const typed = box.value.replace(/\n/g, "");
  if (!typed.trim()) return;
  judge();
  const s = score(cur.text, typed);
  const ms = started ? performance.now() - started : 0;
  const body = { company: cur.company, rcept_no: cur.rcept_no, index: cur.index, target: cur.text, typed, ms, keys, backspaces, live };
  const flash = $("flash");
  try {
    const res = await fetch("/api/practice", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
    if (!res.ok) throw new Error((await res.json()).error);
    done.add(`${cur.rcept_no}:${cur.index}`);
  } catch (err) {
    flash.className = "flash bad";
    flash.textContent = `Couldn't save this sentence: ${err.message}. Is the server still running?`;
    return;
  }
  const left = s.errors.length;
  flash.className = `flash ${left ? "bad" : "good"}`;
  flash.textContent = `${pct(s.accuracy)} · ${Math.round(syllables(typed) / Math.max(ms / 60000, 1 / 60))} syllables/min · ${live.length} made, ${left} left` +
    (left ? ` (${s.errors.slice(0, 4).map(e => `${e.expected || "∅"}→${e.got || "∅"}`).join(", ")}${left > 4 ? " …" : ""})` : "");
  await refreshStats();
  next();
}

function table(rows, head, cell) {
  if (!rows.length) return `<p class="empty">Nothing yet.</p>`;
  return `<table><thead><tr>${head.map(h => `<th>${h}</th>`).join("")}</tr></thead><tbody>${rows.map(cell).join("")}</tbody></table>`;
}
const pairCell = ([k, n]) => {
  const [e, g] = k.split("→");
  return `<tr><td class="pair"><span class="e">${esc(e)}</span> → <span class="g">${esc(g)}</span></td><td class="n">${n}</td></tr>`;
};

async function refreshStats() {
  const res = await fetch("/api/practice/stats");
  if (!res.ok) return;
  const { all, done: list } = await res.json();
  done = new Set(list);
  $("tSent").textContent = all.sentences;
  $("tAcc").textContent = all.sentences ? pct(all.accuracy) : "–";
  $("tCpm").textContent = all.sentences ? Math.round(all.cpm) : "–";
  $("tMade").textContent = all.made;
  $("confusions").innerHTML = table(all.confusions, ["Expected → typed", "Times"], pairCell);
  $("missed").innerHTML = table(all.missed, ["Syllable", "Times"], ([k, n]) => `<tr><td class="pair">${esc(k)}</td><td class="n">${n}</td></tr>`);
  $("left").innerHTML = table(all.leftOver, ["Expected → left as", "Times"], pairCell);
  $("companies").innerHTML = table(Object.entries(all.companies).sort((a, b) => a[1].accuracy - b[1].accuracy), ["Company", "Sentences", "Accuracy"],
    ([k, v]) => `<tr><td>${esc(k)}</td><td class="n">${v.sentences}</td><td class="n">${pct(v.accuracy)}</td></tr>`);
  spark(all.recent);
}

function spark(points) {
  const el = $("spark");
  if (points.length < 2) { el.innerHTML = `<p class="empty">Type two sentences to see a trend.</p>`; return; }
  const W = 600, H = 80, P = 4, best = Math.max(...points.map(p => p.cpm), 1);
  const x = i => P + (i / (points.length - 1)) * (W - 2 * P);
  const yAcc = a => H - P - Math.max(0, (a - 0.5) / 0.5) * (H - 2 * P);
  const yCpm = c => H - P - (c / best) * (H - 2 * P);
  const line = f => points.map((p, i) => `${i ? "L" : "M"}${x(i).toFixed(1)},${f(p).toFixed(1)}`).join("");
  el.innerHTML = `<svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="none" role="img" aria-label="Accuracy and speed over the last ${points.length} sentences">` +
    `<line class="grid-line" x1="0" x2="${W}" y1="${yAcc(1)}" y2="${yAcc(1)}"/><line class="grid-line" x1="0" x2="${W}" y1="${yAcc(0.75)}" y2="${yAcc(0.75)}"/>` +
    `<path class="cpm" d="${line(p => yCpm(p.cpm))}"/><path class="acc" d="${line(p => yAcc(p.accuracy))}"/></svg>`;
}

$("company").addEventListener("change", () => { build(); next(); });
$("order").addEventListener("change", () => { build(); next(); });
$("skipDone").addEventListener("change", () => { build(); next(); });
$("skip").addEventListener("click", () => { next(); });
load();
