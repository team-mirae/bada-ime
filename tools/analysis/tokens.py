"""Token keys: how ambiguous does Korean get when several jamo share one key?

Words (eojeol) are written as 3-set roles (initial / vowel / final). A grouping maps each jamo of a
role to a token. Ambiguity = share of word occurrences whose token string belongs to a more
frequent word (top-1 by word frequency alone, no context: a pessimistic bound for a ranker).
Greedy: repeatedly merge the pair (same role) that adds the least ambiguity.
"""
import collections, sys
from corpus import words, decompose

TOP = int(sys.argv[1]) if len(sys.argv) > 1 else 30000
freq = collections.Counter(w for _, w in words(1_000_000))
total_all = sum(freq.values())
vocab = freq.most_common(TOP)
cover = sum(n for _, n in vocab) / total_all
syms = collections.Counter()
enc = []
for w, n in vocab:
    seq = []
    for ch in w:
        l, v, t = decompose(ch)
        seq += [("i", l), ("v", v), ("f", t or "-")]
    for s in seq: syms[s] += n
    enc.append((seq, n))
syms = list(syms)
idx = {s: i for i, s in enumerate(syms)}
enc = [(tuple(idx[s] for s in seq), n) for seq, n in enc]
total = sum(n for _, n in enc)

def ambiguity(group):
    best = {}; mass = collections.Counter()
    for seq, n in enc:
        key = tuple(group[s] for s in seq)
        mass[key] += n
        if n > best.get(key, 0): best[key] = n
    return 1 - sum(best.values()) / total

group = list(range(len(syms)))
role = [s[0] for s in syms]
def nkeys(r): return len({group[i] for i in range(len(syms)) if role[i] == r and syms[i][1] != "-"})
def members(g): return "".join(syms[i][1] for i in range(len(syms)) if group[i] == g)
print(f"vocab top {TOP} words = {100*cover:.1f}% of running words")
print(f"start: initial {nkeys('i')}, vowel {nkeys('v')}, final {nkeys('f')} keys, ambiguity {100*ambiguity(group):.2f}%")
steps = int(sys.argv[2]) if len(sys.argv) > 2 else 22
snap = []
for step in range(steps):
    best = None
    gs = sorted(set(group))
    for a in gs:
        for b in gs:
            if a >= b: continue
            ia = group.index(a); ib = group.index(b)
            if role[ia] != role[ib] or syms[ia][1] == "-" or syms[ib][1] == "-": continue
            trial = [a if g == b else g for g in group]
            amb = ambiguity(trial)
            if best is None or amb < best[0]: best = (amb, a, b, trial)
    amb, a, b, group = best
    import json
    snap.append({"keys": len(set(g for i, g in enumerate(group) if syms[i][1] != "-")), "groups": {r: sorted(members(g) for g in set(group[i] for i in range(len(syms)) if role[i] == r and syms[i][1] != "-")) for r in "ivf"}})
    json.dump(snap, open("out_tokens.json", "w"), ensure_ascii=False)
    print(f"{step+1:2d}. merge {role[group.index(a)]}: {members(a)}   -> i{nkeys('i')} v{nkeys('v')} f{nkeys('f')}  ambiguity {100*amb:.2f}%", flush=True)
