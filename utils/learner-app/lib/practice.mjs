// Copy practice: cutting filings into sentences, and scoring what was typed against them.
// Pure functions; the server and the page share them (the page loads this file as /practice-lib.js).

/** "II. 사업의 내용" from a DART viewer section page, as plain prose: tables, scripts and tags out. */
export function sectionText(html) {
  let s = html.replace(/<(script|style)\b[\s\S]*?<\/\1>/gi, "");
  s = s.replace(/<table\b[\s\S]*?<\/table>/gi, "\n");
  s = s.replace(/<br\s*\/?>/gi, "\n").replace(/<\/(p|div|li|tr|h\d|span class="xforms_title")>/gi, "\n");
  s = s.replace(/<[^>]+>/g, "");
  s = s.replace(/&nbsp;/g, " ").replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">")
    .replace(/&quot;/g, "\"").replace(/&#39;/g, "'").replace(/&#(\d+);/g, (_, n) => String.fromCodePoint(+n));
  s = s.replace(/\u318D/g, "\u00B7");   // ㆍ (as DART writes the middle dot) → ·, the one bada roman types
  return s.split("\n").map(l => l.replace(/\s+/g, " ").trim()).filter(Boolean).join("\n");
}

const HANGUL = /[가-힣]/g;

/** Sentences worth typing: Korean prose, 12–140 characters, no headings, notes or table remnants. */
export function sentences(text) {
  const out = [];
  const seen = new Set();
  // Lines break mid-sentence in the viewer: join them, then cut after a sentence end.
  const flat = text.split("\n").filter(l => !/^[☞※▶ㆍ\-*]/.test(l) && !/^(\d+|[IVX]+|[가-하]|\(\d+\)|[①-⑳])[.)]\s/.test(l)).join(" ");
  for (let s of flat.split(/(?<=[다요음함됨임])\.\s+/)) {
    s = s.trim().replace(/^(\[[^\]]*\]\s*)+/, "");   // a leading [heading]
    if (!s.endsWith(".")) s += ".";
    const n = s.length, hangul = (s.match(HANGUL) || []).length;
    if (n < 12 || n > 140 || hangul / n < 0.5) continue;
    if (/[|│┃]|\.{3}|_{3}/.test(s) || /\d{4}\.\d{2}\.\d{2}/.test(s)) continue;
    if (seen.has(s)) continue;
    seen.add(s);
    out.push(s);
  }
  return out;
}

/**
 * Aligns typed text to the target, character by character (edit distance), and lists what went wrong.
 * Returns { marks: per target character "ok" | "wrong" | "missed", errors: [{ expected, got, at }],
 * extra: characters typed with no place in the target, accuracy: correct / target length }.
 */
export function score(target, typed) {
  const a = [...target], b = [...typed];
  const n = a.length, m = b.length;
  const d = Array.from({ length: n + 1 }, (_, i) => { const r = new Array(m + 1).fill(0); r[0] = i; return r; });
  for (let j = 0; j <= m; j++) d[0][j] = j;
  for (let i = 1; i <= n; i++)
    for (let j = 1; j <= m; j++)
      d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
  const marks = new Array(n).fill("missed"), errors = [];
  let extra = 0, i = n, j = m;
  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && d[i][j] === d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1)) {
      if (a[i - 1] === b[j - 1]) marks[i - 1] = "ok";
      else { marks[i - 1] = "wrong"; errors.push({ expected: a[i - 1], got: b[j - 1], at: i - 1 }); }
      i--; j--;
    } else if (i > 0 && d[i][j] === d[i - 1][j] + 1) {
      errors.push({ expected: a[i - 1], got: "", at: i - 1 });
      i--;
    } else {
      extra++;
      errors.push({ expected: "", got: b[j - 1], at: i });
      j--;
    }
  }
  errors.reverse();
  const ok = marks.filter(x => x === "ok").length;
  return { marks, errors, extra, accuracy: n ? ok / n : 1 };
}

/** Hangul syllables in a string: the speed unit (characters per minute, as Korean typing tests count). */
export const syllables = s => (s.match(HANGUL) || []).length;

/**
 * Totals over practice records ({ target, typed, ms, backspaces, live: [{expected, got}], company }):
 * accuracy, speed, the mistakes made (live, even if fixed) and left (final), most often first.
 */
export function summarize(records) {
  const count = (list, key) => {
    const m = new Map();
    for (const x of list) { const k = key(x); m.set(k, (m.get(k) || 0) + 1); }
    return [...m.entries()].sort((p, q) => q[1] - p[1]);
  };
  const made = [], left = [], byCompany = new Map();
  let chars = 0, correct = 0, ms = 0, typedSyl = 0, backspaces = 0;
  for (const r of records) {
    const s = score(r.target, r.typed);
    chars += [...r.target].length;
    correct += s.marks.filter(x => x === "ok").length;
    ms += r.ms || 0;
    typedSyl += syllables(r.typed);
    backspaces += r.backspaces || 0;
    made.push(...(r.live || []));
    left.push(...s.errors);
    const c = byCompany.get(r.company) || { sentences: 0, accuracy: 0 };
    c.sentences++; c.accuracy += s.accuracy;
    byCompany.set(r.company, c);
  }
  const pair = e => `${e.expected || "∅"}→${e.got || "∅"}`;
  return {
    sentences: records.length,
    accuracy: chars ? correct / chars : 1,
    cpm: ms ? typedSyl / (ms / 60000) : 0,
    backspaces,
    made: made.length,
    left: left.length,
    confusions: count(made.filter(e => e.expected && e.got), pair).slice(0, 20),
    missed: count([...made, ...left].filter(e => e.expected), e => e.expected).slice(0, 20),
    leftOver: count(left, pair).slice(0, 20),
    companies: Object.fromEntries([...byCompany].map(([k, v]) => [k, { sentences: v.sentences, accuracy: v.accuracy / v.sentences }])),
    recent: records.slice(-30).map(r => ({ accuracy: score(r.target, r.typed).accuracy, cpm: r.ms ? syllables(r.typed) / (r.ms / 60000) : 0 })),
  };
}
