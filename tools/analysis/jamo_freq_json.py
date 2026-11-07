"""Jamo frequencies for the layout editor: per syllable, by role, plus standalone jamo (ㅋㅋ)."""
import collections, json, os, re
from corpus import ROOT, SOURCES, decompose, L, V, T
ini, med, fin, alone = (collections.Counter() for _ in range(4))
syl = 0; PER = 3_000_000
JAMO = re.compile(r"[ㄱ-ㆎ]")
SYL = re.compile(r"[가-힣]")
for name, rel in SOURCES.items():
    n = 0
    with open(os.path.join(ROOT, rel), encoding="utf-8", errors="ignore") as f:
        for line in f:
            for ch in SYL.findall(line):
                l, v, t = decompose(ch); ini[l] += 1; med[v] += 1; syl += 1; n += 1
                if t: fin[t] += 1
            for j in JAMO.findall(line): alone[j] += 1
            if n >= PER: break
pct = lambda c: {k: round(100 * v / syl, 4) for k, v in c.most_common()}
out = dict(source=f"{syl} Hangul syllables, 3M each from news, petitions, NSMC reviews, conversation (jis-moonshot corpora)",
           unit="% of syllables", initial=pct(ini), medial=pct(med), final=pct(fin), standalone=pct(alone),
           no_final=round(100 * (syl - sum(fin.values())) / syl, 2))
json.dump(out, open("out_jamo_freq.json", "w"), ensure_ascii=False)
print(json.dumps(out, ensure_ascii=False)[:1500])
