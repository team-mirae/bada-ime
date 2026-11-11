// Downloads the practice corpus: "II. 사업의 내용" of each filing in corpus/samsung-filings.json, from the
// public DART viewer (dart.fss.or.kr), cut into sentences. Writes data/corpus/samsung.json (git-ignored).
// Usage: node tools/fetch-dart.mjs
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { sectionText, sentences } from "../lib/practice.mjs";

const HERE = fileURLToPath(new URL("..", import.meta.url));
const LIST = JSON.parse(readFileSync(join(HERE, "corpus", "samsung-filings.json"), "utf8"));
const OUT = join(HERE, "data", "corpus");
const DART = "https://dart.fss.or.kr";
const sleep = ms => new Promise(r => setTimeout(r, ms));

async function get(url) {
  const res = await fetch(url, { headers: { "user-agent": "bada-learner-app (corpus fetch)" } });
  if (!res.ok) throw new Error(`${res.status} ${url}`);
  return res.text();
}

/** The viewer's table of contents entry for "II. 사업의 내용": the parameters of its section page. */
function sectionParams(main) {
  const at = main.indexOf("['text'] = \"II. 사업의 내용\"");
  if (at < 0) return null;
  const block = main.slice(at, at + 800);
  const field = k => (block.match(new RegExp(`\\['${k}'\\]\\s*=\\s*"([^"]*)"`)) || [])[1];
  const p = { rcpNo: field("rcpNo"), dcmNo: field("dcmNo"), eleId: field("eleId"), offset: field("offset"), length: field("length"), dtd: field("dtd") };
  return Object.values(p).every(Boolean) ? p : null;
}

const companies = [];
for (const f of LIST.filings) {
  const viewer = `${DART}/dsaf001/main.do?rcpNo=${f.rcept_no}`;
  try {
    const params = sectionParams(await get(viewer));
    if (!params) throw new Error("no 'II. 사업의 내용' in the table of contents");
    await sleep(400);
    const html = await get(`${DART}/report/viewer.do?${new URLSearchParams(params)}`);
    const list = sentences(sectionText(html));
    companies.push({ company: f.company, corp_code: f.corp_code, rcept_no: f.rcept_no, url: viewer, sentences: list });
    console.log(`${f.company}: ${list.length} sentences`);
  } catch (e) {
    console.error(`${f.company}: ${e.message}`);
  }
  await sleep(400);
}
mkdirSync(OUT, { recursive: true });
const file = join(OUT, "samsung.json");
writeFileSync(file, JSON.stringify({ source: "DART (dart.fss.or.kr), 'II. 사업의 내용'", report: LIST.report, fetched: new Date().toISOString(), companies }, null, 1) + "\n");
console.log(`${companies.reduce((n, c) => n + c.sentences.length, 0)} sentences from ${companies.length} companies → ${file}`);
