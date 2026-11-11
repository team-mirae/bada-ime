// Loads the engine for a page: the manifest (web/build.mjs), the layouts, the wasm and the data,
// with progress. Shared by the simulator (app.js) and Learn Korean (learn.js).
import { Engine, DATA_FILES } from "./engine.js";

async function fetchBytes(url, gz, onBytes) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  let body = res.body.pipeThrough(new TransformStream({ transform(chunk, c) { onBytes(chunk.length); c.enqueue(chunk); } }));
  if (gz) body = body.pipeThrough(new DecompressionStream("gzip"));
  return new Uint8Array(await new Response(body).arrayBuffer());
}

/** `onProgress(got, total)` in bytes; then "starting" once everything is in. */
export async function loadEngine(onProgress = () => {}) {
  const manifest = await (await fetch("manifest.json", { cache: "no-cache" })).json();
  const layouts = {};
  await Promise.all(manifest.layouts.map(async path => {
    const l = await (await fetch(path)).json();
    layouts[l.id] = l;
  }));
  const total = manifest.wasm.size + DATA_FILES.reduce((n, f) => n + (manifest.data[f]?.size || 0), 0);
  let got = 0;
  const tick = n => { got += n; onProgress(got, total); };
  const [wasm, ...files] = await Promise.all([
    fetchBytes(manifest.wasm.path, false, tick),
    ...DATA_FILES.map(f => manifest.data[f] ? fetchBytes(manifest.data[f].path, true, tick) : Promise.resolve(new Uint8Array())),
  ]);
  onProgress(total, total, "starting");
  await new Promise(r => setTimeout(r, 30));
  const engine = await Engine.load(wasm);
  files.forEach((bytes, i) => engine.put(i, bytes));
  engine.init();
  return { engine, manifest, layouts };
}
