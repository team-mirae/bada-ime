"""Score a bada-layout/1 JSON (from the editor) on the corpora, effort model v2.

Where a jamo has several keys, the cheapest route through the whole word is taken (Viterbi: a
typist who plans ahead). Jamo with no key fall back to 2-set sequences (ㅘ = ㅗ ㅏ,
ㄺ = ㄹ ㄱ). Initial ㅇ is skipped where `implied_ieung` allows (word start / after an open
syllable, 2-set rule). Ambiguity isn't scored here.
"""
import collections, json, sys
import numpy as np
import optimize48 as om
from corpus import words, decompose
from geometry import KEYS
HOME = {"KeyA","KeyS","KeyD","KeyF","KeyJ","KeyK","KeyL","Semicolon"}
base = np.array([0.0 if k[0] in HOME else k[4] for k in KEYS])
code_i = {k[0]: i for i, k in enumerate(KEYS)}
ROLES = {"i": {"i", "c", "j"}, "v": {"v", "j"}, "f": {"f", "c", "j"}}

def load(path):
    d = json.load(open(path)); where = collections.defaultdict(list)
    for code, v in d["keys"].items():
        for ly in ("base", "shift"):
            for t in v.get(ly) or []:
                for role, ok in ROLES.items():
                    if t["r"] in ok: where[(role, t["t"])].append((code_i[code], ly == "shift"))
    return d, where

def jamo_seq(role, j, where):
    if (role, j) in where: return [(role, j)]
    if role == "v" and j in om.VSEQ: return [x for p in om.VSEQ[j] for x in jamo_seq("v", p, where)]
    if role == "f" and j in om.CLUSTER: return [x for p in om.CLUSTER[j] for x in jamo_seq("f", p, where)]
    raise KeyError(f"no key for {role} {j}")

def score(path, per=1_000_000):
    d, where = load(path)
    sample = collections.Counter(w for _, w in words(per))
    tot = eff = sfb = same = bis = shifts = syl = 0; fing = collections.Counter(); rest = 0
    missing = collections.Counter()
    for w, c in sample.items():
        prev = None; prev_syl = None; units = []
        for ch in w:
            l, v, t = decompose(ch); syl += c
            seq = []
            if not (l == "ㅇ" and d.get("implied_ieung") and (prev_syl is None or not prev_syl[2])):
                seq += jamo_seq("i", l, where)
            seq += jamo_seq("v", v, where)
            if t: seq += jamo_seq("f", t, where)
            units += seq
            prev_syl = (l, v, t)
        # Viterbi: the cheapest route through the word's duplicates (a typist who plans ahead)
        layers = [where[u] for u in units]
        costs = [base[k] + (om.SHIFT_COST if sh else 0) for k, sh in layers[0]]; back = [[None]*len(layers[0])]
        for li in range(1, len(layers)):
            nc, nb = [], []
            for k, sh in layers[li]:
                e = base[k] + (om.SHIFT_COST if sh else 0)
                best = min(range(len(layers[li-1])), key=lambda j: costs[j] + om.P[layers[li-1][j][0], k])
                nc.append(costs[best] + om.P[layers[li-1][best][0], k] + e); nb.append(best)
            costs, back = nc, back + [nb]
        j = min(range(len(costs)), key=costs.__getitem__); eff += c * costs[j]; path = []
        for li in range(len(layers) - 1, -1, -1):
            path.append(layers[li][j]); j = back[li][j] if back[li][j] is not None else j
        path.reverse()
        for k, sh in path:
            tot += c; shifts += c * sh
            f = KEYS[k][3]; fing[f] += c; rest += c * (KEYS[k][0] in HOME)
            if prev is not None:
                bis += c; pf = KEYS[prev][3]
                if pf == f and prev != k: sfb += c
                if (pf >= 4) == (f >= 4): same += c
            prev = k
    return dict(name=d.get("name"), keys_per_syl=round(tot/syl, 3), effort_per_syl=round(eff/syl, 3),
                shift_pct=round(100*shifts/tot, 2), resting_keys_pct=round(100*rest/tot, 1),
                left_pct=round(100*sum(fing[f] for f in range(4))/tot, 1), pinky_pct=round(100*(fing[0]+fing[7])/tot, 1),
                same_hand_pct=round(100*same/bis, 1), sfb_pct=round(100*sfb/bis, 2))

if __name__ == "__main__":
    for p in sys.argv[1:]: print(json.dumps(score(p), ensure_ascii=False))
