// Builds the web simulator into web/dist: the engine as WebAssembly (web/wasm), the page (web/src),
// the keyboards (tools/render/keyboards.js), the layouts (docs/layouts/*.json) and the engine's data
// (engine/data, gzipped; the page unzips it). Data and wasm are named by content hash, so they can
// be cached for good; manifest.json says where they are.
//
//   node web/build.mjs            then serve web/dist (e.g. python3 -m http.server -d web/dist)
//   node web/build.mjs --vercel   also web/.vercel/output (Vercel's Build Output API), for
//                                 `vercel deploy --prebuilt` from web/ (.github/workflows/deploy-web.yml)
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, cpSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";
import { DATA_FILES } from "./src/engine.js";

const web = fileURLToPath(new URL(".", import.meta.url));
const root = web + "../";
const out = web + "dist/";
// Data and wasm are named by their hash: cached for good. The manifest says which they are: never cached.
const HEADERS = [
  ["^/data/(.*)$", "public, max-age=31536000, immutable"],
  ["^/(.*)\\.wasm$", "public, max-age=31536000, immutable"],
  ["^/manifest\\.json$", "no-cache"],
];
const hash = bytes => createHash("sha256").update(bytes).digest("hex").slice(0, 12);

execFileSync("cargo", ["build", "--release", "--target", "wasm32-unknown-unknown", "--manifest-path", web + "wasm/Cargo.toml"], { stdio: "inherit" });

rmSync(out, { recursive: true, force: true });
for (const dir of ["data", "layouts", "licenses"]) mkdirSync(out + dir, { recursive: true });
for (const f of readdirSync(web + "src")) copyFileSync(web + "src/" + f, out + f);
copyFileSync(root + "tools/render/keyboards.js", out + "keyboards.js");
copyFileSync(root + "docs/images/icon/bada-icon.svg", out + "icon.svg");

const wasm = readFileSync(web + "wasm/target/wasm32-unknown-unknown/release/bada_web.wasm");
const version = readFileSync(root + "engine/Cargo.toml", "utf8").match(/^version = "(.+)"/m)[1];
const manifest = { version, wasm: { path: `bada.${hash(wasm)}.wasm`, size: wasm.length }, data: {}, layouts: [] };
writeFileSync(out + manifest.wasm.path, wasm);

for (const f of DATA_FILES) {
  const gz = gzipSync(readFileSync(root + "engine/data/" + f), { level: 9 });
  const path = `data/${f}.${hash(gz)}.gz`;
  writeFileSync(out + path, gz);
  manifest.data[f] = { path, size: gz.length };
}
for (const f of readdirSync(root + "engine/data").filter(f => f.startsWith("LICENSE"))) {
  copyFileSync(root + "engine/data/" + f, out + "licenses/" + f);
}
copyFileSync(root + "LICENSE", out + "licenses/LICENSE.txt");

for (const f of readdirSync(root + "docs/layouts").filter(f => f.endsWith(".json")).sort()) {
  const bytes = readFileSync(root + "docs/layouts/" + f);
  writeFileSync(out + "layouts/" + f, bytes);
  manifest.layouts.push(`layouts/${f}?v=${hash(bytes)}`);
}
writeFileSync(out + "manifest.json", JSON.stringify(manifest, null, 1));
const total = manifest.wasm.size + Object.values(manifest.data).reduce((n, d) => n + d.size, 0);
console.log(`web/dist: ${manifest.layouts.length} layouts, ${(total / 1e6).toFixed(1)} MB to download`);

if (process.argv.includes("--vercel")) {
  const vercel = web + ".vercel/output/";
  rmSync(vercel, { recursive: true, force: true });
  cpSync(out, vercel + "static", { recursive: true });
  const routes = [
    ...HEADERS.map(([src, value]) => ({ src, headers: { "Cache-Control": value }, continue: true })),
    { handle: "filesystem" },
  ];
  writeFileSync(vercel + "config.json", JSON.stringify({ version: 3, routes }, null, 1));
  console.log("web/.vercel/output: ready for vercel deploy --prebuilt");
}
