"""Re-score the greedy groupings (out_tokens.json) on a bigger vocabulary, top-1 and top-3."""
import collections, json, sys
from corpus import words, decompose
TOP = int(sys.argv[1]) if len(sys.argv) > 1 else 100000
freq = collections.Counter(w for _, w in words(1_000_000))
vocab = freq.most_common(TOP); total = sum(n for _, n in vocab)
print(f"top {TOP} words = {100*total/sum(freq.values()):.1f}% of running words")
dec = [([decompose(ch) for ch in w], n) for w, n in vocab]
for k, snap in enumerate(json.load(open("out_tokens.json"))):
    if k % 4 != 3 and k != len(json.load(open("out_tokens.json"))) - 1: continue
    m = {}
    for r, gs in snap["groups"].items():
        for gi, g in enumerate(gs):
            for ch in g: m[(r, ch)] = f"{r}{gi}"
    buckets = collections.defaultdict(list)
    for d, n in dec:
        key = tuple(x for l, v, t in d for x in (m.get(("i", l), l), m.get(("v", v), v), m.get(("f", t), t) if t else "-"))
        buckets[key].append(n)
    top1 = sum(max(b) for b in buckets.values()) / total
    top3 = sum(sum(sorted(b, reverse=True)[:3]) for b in buckets.values()) / total
    print(f"step {k+1:2d}: {snap['keys']:2d} keys  wrong at top-1 {100*(1-top1):5.2f}%  not in top 3 {100*(1-top3):5.2f}%")
