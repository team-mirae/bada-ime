"""Place token groupings (out_tokens.json) on the 48 kana-space keys, 1KRO, no Shift.

Effort model v2 (owner, 2026-11-07): A S D F J K L ; cost zero (the fingers rest there).
Initial ㅇ is always implied: every vowel token is one key, so a vowel after a vowel or a final
always starts a new syllable.
"""
import collections, json, sys
import numpy as np
import optimize48 as om
from corpus import words, decompose
from geometry import KEYS

HOME = {"KeyA","KeyS","KeyD","KeyF","KeyJ","KeyK","KeyL","Semicolon"}
base = np.array([0.0 if k[0] in HOME else k[4] for k in KEYS])
om.eff = np.array([base[k] + (100.0 if sh else 0.0) for k, sh in om.SLOTS])   # Shift slots unused

snaps = json.load(open("out_tokens.json"))
sample = list(collections.Counter(w for _, w in words(1_000_000)).items())
ambig = {s["keys"]: a for s, a in []}
pick = [int(x) for x in sys.argv[1:]] or [20, 28, 32, 36, 40]
results = []
for step in pick:
    snap = snaps[step - 1]
    tok = {}
    for r, gs in snap["groups"].items():
        for g in gs:
            if r == "i" and g == "ㅇ": continue
            for ch in g: tok[(r, ch)] = f"{r}:{g}"
    units = sorted(set(tok.values())); idx = {u: i for i, u in enumerate(units)}; n = len(units)
    U = np.zeros(n); B = np.zeros((n, n)); syl = 0
    for w, c in sample:
        ks = []
        for ch in w:
            l, v, t = decompose(ch); syl += c
            if l != "ㅇ": ks.append(idx[tok[("i", l)]])
            ks += [idx[tok[("v", x)]] for x in ([v] if ("v", v) in tok else om.VSEQ[v])]
            if t:
                if ("f", t) in tok: ks.append(idx[tok[("f", t)]])
                else: ks += [idx[tok[("f", x)]] for x in om.CLUSTER[t]]   # rare double final: two keys
        for k in ks: U[k] += c
        for a, b in zip(ks, ks[1:]): B[a, b] += c
    assign, cst = min((om.anneal(U, B, iters=120000, seed=k) for k in range(3)), key=lambda r: r[1])
    st = om.report(None, assign, U, B, syl)
    st.pop("shift_pct", None)
    home = sum(U[i] for i in range(n) if KEYS[om.key_of[assign[i]]][0] in HOME) / U.sum()
    st["on_resting_keys_pct"] = round(100 * home, 1)
    lay = {}
    for u, a, cnt in zip(units, assign, U):
        code = KEYS[om.key_of[a]][0]; r, g = u.split(":")
        lay[code] = {"base": {"t": g, "c": r, "f": round(float(cnt), 1)}}
    print(f"\n### step {step}: {n} token keys\n{json.dumps(st, ensure_ascii=False)}", flush=True)
    rows = [[k[0] for k in KEYS if k[1] == r] for r in range(4)]
    for r, row in enumerate(rows):
        print("  " * r + " ".join(f"{lay[c]['base']['t'][:3]:<3}" if c in lay else " · " for c in row))
    results.append(dict(name=f"tokens {n} keys (step {step})", stats=st, layout=lay))
json.dump(results, open("out_place_tokens.json", "w"), ensure_ascii=False, indent=1)
