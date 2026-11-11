// bada learner-app: a local page with a plain textarea. Type with bada ime (the system input method);
// every key, composition and input event is logged to data/sessions/, on this Mac only.
// Usage: node server.mjs [--port 5713]
import { createServer } from "node:http";
import { appendFileSync, copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, renameSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { learn, parseSession, toDictionary } from "./lib/learn.mjs";
import { RENDER_DIR } from "./lib/layouts.mjs";
import { summarize } from "./lib/practice.mjs";

const HERE = fileURLToPath(new URL(".", import.meta.url));
const DATA = join(HERE, "data"), SESSIONS = join(DATA, "sessions"), EXPORTS = join(DATA, "exports");
const CORPUS = join(DATA, "corpus", "samsung.json"), PRACTICE = join(DATA, "practice");
const BADA_DIR = join(homedir(), "Library", "Application Support", "bada-ime");
const PORT = Number(process.argv[process.argv.indexOf("--port") + 1]) || 5713;
mkdirSync(SESSIONS, { recursive: true });
mkdirSync(EXPORTS, { recursive: true });
mkdirSync(PRACTICE, { recursive: true });
const RESULTS = join(PRACTICE, "results.jsonl");
const results = () => existsSync(RESULTS)
  ? readFileSync(RESULTS, "utf8").split("\n").filter(Boolean).map(l => JSON.parse(l)) : [];

const STATIC = {
  "/": join(HERE, "public", "index.html"),
  "/app.js": join(HERE, "public", "app.js"),
  "/practice": join(HERE, "public", "practice.html"),
  "/practice.js": join(HERE, "public", "practice.js"),
  "/practice-lib.js": join(HERE, "lib", "practice.mjs"),
  "/keyboards.js": join(RENDER_DIR, "keyboards.js"),
  "/layouts.js": join(RENDER_DIR, "layouts.js"),
};
const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".mjs": "text/javascript; charset=utf-8", ".json": "application/json; charset=utf-8" };
const SESSION_ID = /^[0-9TZ-]+-[a-z0-9]{4}$/;
const stamp = () => new Date().toISOString().replace(/[:.]/g, "-");

function send(res, status, body, type = TYPES[".json"], extra = {}) {
  res.writeHead(status, { "content-type": type, "cache-control": "no-store", ...extra });
  res.end(typeof body === "string" || Buffer.isBuffer(body) ? body : JSON.stringify(body, null, 2));
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let size = 0; const chunks = [];
    req.on("data", c => { size += c.length; if (size > 8 << 20) { reject(new Error("too large")); req.destroy(); } else chunks.push(c); });
    req.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
    req.on("error", reject);
  });
}

function allSessions(only) {
  return readdirSync(SESSIONS).filter(f => f.endsWith(".jsonl") && (!only || f === only + ".jsonl")).sort()
    .map(f => parseSession(readFileSync(join(SESSIONS, f), "utf8")));
}

function build(url) {
  const sessions = allSessions(url.searchParams.get("session"));
  const exp = learn(sessions, { texts: url.searchParams.get("texts") === "1" });
  return { exp, min: Math.max(1, Number(url.searchParams.get("min")) || 2) };
}

const server = createServer(async (req, res) => {
  // Local only: refuse other hosts (DNS rebinding) and cross-site writes.
  const host = (req.headers.host || "").replace(/:\d+$/, "");
  if (!["127.0.0.1", "localhost"].includes(host)) return send(res, 403, { error: "local only" });
  if (req.method === "POST" && req.headers.origin && !/^http:\/\/(127\.0\.0\.1|localhost):\d+$/.test(req.headers.origin))
    return send(res, 403, { error: "cross-site" });
  const url = new URL(req.url, `http://${req.headers.host}`);
  try {
    if (req.method === "GET" && STATIC[url.pathname]) {
      const p = STATIC[url.pathname];
      return send(res, 200, readFileSync(p), TYPES[extname(p)]);
    }
    if (req.method === "POST" && url.pathname === "/api/events") {
      const { session, events } = JSON.parse(await readBody(req));
      if (!SESSION_ID.test(session) || !Array.isArray(events)) return send(res, 400, { error: "bad session or events" });
      appendFileSync(join(SESSIONS, session + ".jsonl"), events.map(e => JSON.stringify(e) + "\n").join(""));
      return send(res, 200, { ok: true, saved: events.length });
    }
    // Copy practice (/practice): the corpus, one result per sentence typed, and their totals.
    if (req.method === "GET" && url.pathname === "/api/corpus") {
      if (!existsSync(CORPUS)) return send(res, 404, { error: "no corpus: run node tools/fetch-dart.mjs" });
      return send(res, 200, readFileSync(CORPUS));
    }
    if (req.method === "POST" && url.pathname === "/api/practice") {
      const r = JSON.parse(await readBody(req));
      if (typeof r.target !== "string" || typeof r.typed !== "string") return send(res, 400, { error: "target and typed are required" });
      const keep = { t: new Date().toISOString(), company: String(r.company || ""), rcept_no: String(r.rcept_no || ""), index: Number(r.index) || 0,
        target: r.target, typed: r.typed, ms: Number(r.ms) || 0, keys: Number(r.keys) || 0, backspaces: Number(r.backspaces) || 0,
        live: Array.isArray(r.live) ? r.live.slice(0, 500).map(e => ({ expected: String(e.expected || ""), got: String(e.got || "") })) : [] };
      appendFileSync(RESULTS, JSON.stringify(keep) + "\n");
      return send(res, 200, { ok: true });
    }
    if (req.method === "GET" && url.pathname === "/api/practice/stats") {
      const all = results(), company = url.searchParams.get("company");
      return send(res, 200, { all: summarize(all), company: company ? summarize(all.filter(r => r.company === company)) : null,
        done: all.map(r => `${r.rcept_no}:${r.index}`) });
    }
    if (req.method === "GET" && url.pathname === "/api/stats") {
      const { exp } = build(url);
      return send(res, 200, {
        sessions: exp.sessions, totals: exp.totals,
        words: Object.fromEntries(Object.entries(exp.words).map(([k, v]) => [k, v.length])),
        top: exp.words.korean.slice(0, 12), rejected: exp.rejected.slice(0, 8),
      });
    }
    if (req.method === "GET" && (url.pathname === "/api/export/training" || url.pathname === "/api/export/dictionary")) {
      const { exp, min } = build(url);
      const dictionary = url.pathname.endsWith("dictionary");
      const body = dictionary ? toDictionary(exp, { min }) : exp;
      const name = `${dictionary ? "dictionary" : "training"}-${stamp()}.json`;
      writeFileSync(join(EXPORTS, name), JSON.stringify(body, null, 2) + "\n");
      return send(res, 200, body, TYPES[".json"], { "content-disposition": `attachment; filename="${name}"`, "x-saved-as": join(EXPORTS, name) });
    }
    if (req.method === "POST" && url.pathname === "/api/install") {
      // Plug in directly (not recommended): replaces dictionary.json, keeping the old one beside it.
      const { confirm, min } = JSON.parse(await readBody(req) || "{}");
      if (confirm !== "replace") return send(res, 400, { error: "confirm must be \"replace\"" });
      const exp = learn(allSessions());
      const dict = toDictionary(exp, { min: Math.max(1, Number(min) || 2) });
      mkdirSync(BADA_DIR, { recursive: true });
      const target = join(BADA_DIR, "dictionary.json");
      let backup = null;
      if (existsSync(target)) {
        backup = join(BADA_DIR, `dictionary.backup-${stamp()}.json`);
        copyFileSync(target, backup);
      }
      const tmp = target + ".tmp";
      writeFileSync(tmp, JSON.stringify(dict, null, 2) + "\n");
      renameSync(tmp, target);
      return send(res, 200, { ok: true, path: target, backup, positive: dict.positive.length });
    }
    send(res, 404, { error: "not found" });
  } catch (e) {
    send(res, 500, { error: String(e.message || e) });
  }
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(`bada learner-app: http://127.0.0.1:${PORT}/`);
  console.log(`logging to ${SESSIONS}`);
});
