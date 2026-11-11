import { test } from "node:test";
import assert from "node:assert/strict";
import { score, sectionText, sentences, summarize, syllables } from "../lib/practice.mjs";

test("score aligns and names the mistakes", () => {
  const s = score("우리는 한국", "우라는 한국");
  assert.deepEqual(s.errors, [{ expected: "리", got: "라", at: 1 }]);
  assert.equal(s.marks[1], "wrong");
  assert.equal(Math.round(s.accuracy * 6), 5);
  // a dropped syllable is missed, not a cascade of wrong ones
  const d = score("삼성전자", "삼전자");
  assert.deepEqual(d.errors, [{ expected: "성", got: "", at: 1 }]);
  assert.deepEqual(d.marks, ["ok", "missed", "ok", "ok"]);
  assert.equal(score("가", "가나").extra, 1);
});

test("sentences keep Korean prose only", () => {
  const html = `<p>II. 사업의 내용</p><p>1. 사업의 개요</p><p>당사는 전자 제품을 생산하고 판매하고</p><p>있습니다. 짧다.</p>
    <table><tr><td>표 안의 문장은 빼야 합니다.</td></tr></table><p>☞ 자세한 사항은 참고하시기 바랍니다.</p>`;
  assert.deepEqual(sentences(sectionText(html)), ["당사는 전자 제품을 생산하고 판매하고 있습니다."]);
});

test("summarize counts mistakes made and left", () => {
  const r = [
    { company: "A", target: "우리는", typed: "우리는", ms: 6000, live: [{ expected: "리", got: "라" }] },
    { company: "A", target: "한국", typed: "한극", ms: 6000, live: [{ expected: "국", got: "극" }] },
  ];
  const s = summarize(r);
  assert.equal(s.sentences, 2);
  assert.equal(s.made, 2);
  assert.equal(s.left, 1);
  assert.deepEqual(s.leftOver, [["국→극", 1]]);
  assert.equal(s.cpm, syllables("우리는한극") / (12000 / 60000));
});
