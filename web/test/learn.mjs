// Types every Learn Korean drill (web/src/learn-content.js) through the engine: each item's keys,
// then Space, must commit its word, as the top reading or (for the words a lesson lists in `menu`)
// from the menu.
//   node web/test/learn.mjs   (after node web/build.mjs)
import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { Engine, DATA_FILES } from "../src/engine.js";
import { LESSONS, lessonItems, twoSet, ANTHEM } from "../src/learn-content.js";
import { keyOf } from "../src/learn-keys.js";

const root = fileURLToPath(new URL("../../", import.meta.url));
const engine = await Engine.load(readFileSync(root + "web/wasm/target/wasm32-unknown-unknown/release/bada_web.wasm"));
DATA_FILES.forEach((f, i) => existsSync(root + "engine/data/" + f) && engine.put(i, readFileSync(root + "engine/data/" + f)));
engine.init();

assert.equal(twoSet("한국"), "gksrnr");
assert.equal(twoSet("닭"), "ekfr");
assert.equal(ANTHEM.lines.length, 4);

let checked = 0;
const failures = [];
for (const lesson of LESSONS) {
  engine.applySettings({ enabled: [lesson.level.layout] });
  engine.setMode("henkan");
  for (const item of lessonItems(lesson)) {
    engine.flush();
    let last = null;
    for (const ch of item.keys) {
      const { code, shift } = keyOf(ch);
      last = engine.key(code, { shift });
    }
    const top = last?.preedit ?? "";
    const ok = top === item.ko || (lesson.menu?.includes(item.ko) && last.menu.includes(item.ko));
    if (!ok) failures.push(`${lesson.id} ${item.keys} → ${top} (want ${item.ko}; menu ${last?.menu.join(" ")})`);
    engine.flush();
    checked++;
  }
}
assert.deepEqual(failures, [], failures.join("\n"));
console.log(`learn: ${checked} drill items type as written`);
