"""Score keystroke systems for Korean on the 26 JIS letter keys, and place their units.

A *system* says which units get a key (jamo, compounds, tense consonants), which are typed as
sequences, and whether initial ㅇ is implied. The text is turned into unit streams; simulated
annealing places the units on the letter keys under a simple effort model.
"""
import collections, json, math, random, sys
import numpy as np
from corpus import words, decompose

# --- keyboard: 26 letter keys, (row, col) with col 0-9 left pinky .. right pinky
KEYS = "QWERTYUIOPASDFGHJKLZXCVBNM"
POS = [(0,c) for c in range(10)] + [(1,c) for c in range(9)] + [(2,c) for c in range(7)]
FINGER = [0,1,2,3,3,4,4,5,6,7]           # col -> finger (0 L pinky .. 7 R pinky)
HAND = lambda f: 0 if f < 4 else 1
EFF = {0:[2.0,1.5,1.2,1.3,1.8,2.0,1.3,1.2,1.5,2.0],
       1:[1.3,1.0,0.8,0.7,1.4,1.4,0.7,0.8,1.0],
       2:[2.2,1.9,1.5,1.4,1.8,1.5,1.4]}
eff = np.array([EFF[r][c] for r, c in POS])
fin = [FINGER[c] for _, c in POS]
SHIFT_COST = 2.0
N = len(KEYS)
P = np.zeros((N, N))
for i,(ri,ci) in enumerate(POS):
    for j,(rj,cj) in enumerate(POS):
        fi, fj = fin[i], fin[j]
        if i == j: P[i,j] = 0.3
        elif fi == fj: P[i,j] = 2.5 + (1.0 if abs(ri-rj) == 2 else 0)
        elif HAND(fi) == HAND(fj):
            P[i,j] = 0.3 + (0.8 if abs(ri-rj) == 2 else 0)
            inward = (fj > fi) if HAND(fi) == 0 else (fj < fi)
            if inward and abs(fi-fj) == 1 and ri == rj: P[i,j] -= 0.2

# --- systems
CONS = list("ㄱㄴㄷㄹㅁㅂㅅㅇㅈㅊㅋㅌㅍㅎ")
CLUSTER = {"ㄳ":"ㄱㅅ","ㄵ":"ㄴㅈ","ㄶ":"ㄴㅎ","ㄺ":"ㄹㄱ","ㄻ":"ㄹㅁ","ㄼ":"ㄹㅂ","ㄽ":"ㄹㅅ","ㄾ":"ㄹㅌ","ㄿ":"ㄹㅍ","ㅀ":"ㄹㅎ","ㅄ":"ㅂㅅ"}
TENSE = {"ㄲ":"ㄱ","ㄸ":"ㄷ","ㅃ":"ㅂ","ㅆ":"ㅅ","ㅉ":"ㅈ"}
# how each vowel is typed if it has no key of its own, in preference order
VSEQ = {"ㅘ":"ㅗㅏ","ㅙ":"ㅗㅐ","ㅚ":"ㅗㅣ","ㅝ":"ㅜㅓ","ㅞ":"ㅜㅔ","ㅟ":"ㅜㅣ","ㅢ":"ㅡㅣ",
        "ㅑ":"ㅣㅏ","ㅕ":"ㅣㅓ","ㅛ":"ㅣㅗ","ㅠ":"ㅣㅜ","ㅒ":"ㅑㅣ","ㅖ":"ㅕㅣ","ㅐ":"ㅏㅣ","ㅔ":"ㅓㅣ"}

def make_system(name, vowels, extra, implied, shifted_vowels=()):
    units = CONS + list(vowels) + list(extra)
    assert len(units) == N, (name, len(units))
    return dict(name=name, units=units, keyed=set(units), implied=implied, shifted_vowels=set(shifted_vowels))

def type_vowel(v, s):
    if v in s["keyed"]: return [v]
    if v in s["shifted_vowels"]: return ["⇧" + {"ㅒ":"ㅐ","ㅖ":"ㅔ"}[v]]
    out = []
    for p in VSEQ[v]: out += type_vowel(p, s)
    return out

def type_cons(c, s):
    if c in s["keyed"]: return [c]
    if c in TENSE: return ["⇧" + TENSE[c]] if "⇧"+c not in s.get("shift_units", ()) else ["⇧"+c]
    return [u for p in CLUSTER[c] for u in type_cons(p, s)]

def compounds(s):
    """Vowel pairs the automaton joins: any sequence used to type a vowel."""
    pairs = set()
    for v in VSEQ:
        seq = type_vowel(v, s)
        if len(seq) > 1:
            for a, b in zip(seq, seq[1:]): pairs.add((a, b))
    return pairs

def stream(s, sample):
    comp = compounds(s)
    uni = collections.Counter(); bi = collections.Counter(); syl = 0; shifts = 0
    for w, n in sample:
        keys = []; prev = None
        for ch in w:
            l, v, t = decompose(ch); syl += n
            vk = type_vowel(v, s)
            if l == "ㅇ" and s["implied"] and (prev is None or not prev[2]) and \
               not (prev and (prev[3][-1], vk[0]) in comp):
                pass
            else:
                keys += type_cons(l, s)
            keys += vk
            if t: keys += type_cons(t, s)
            prev = (l, v, t, vk)
        for k in keys:
            uni[k] += n
            if k.startswith("⇧"): shifts += n
        for a, b in zip(keys, keys[1:]): bi[(a.lstrip("⇧"), b.lstrip("⇧"))] += n
    return uni, bi, syl, shifts

def matrices(s, uni, bi):
    idx = {u:i for i,u in enumerate(s["units"])}
    U = np.zeros(N); B = np.zeros((N, N)); sh = 0
    for k, n in uni.items():
        U[idx[k.lstrip("⇧")]] += n
        if k.startswith("⇧"): sh += n
    for (a, b), n in bi.items(): B[idx[a], idx[b]] += n
    return U, B, sh

def cost(perm, U, B, sh):          # perm[unit] = key position
    return U @ eff[perm] + (B * P[np.ix_(perm, perm)]).sum() + sh * SHIFT_COST

def anneal(U, B, sh, fixed=None, iters=60000, seed=0, constraint=None):
    rnd = random.Random(seed)
    perm = np.array(fixed if fixed is not None else rnd.sample(range(N), N))
    if fixed is not None: return perm, cost(perm, U, B, sh)
    cur = cost(perm, U, B, sh); best = (cur, perm.copy()); T0 = cur * 0.02
    for it in range(iters):
        T = T0 * (1 - it/iters) + 1e-9
        i, j = rnd.randrange(N), rnd.randrange(N)
        if i == j: continue
        perm[i], perm[j] = perm[j], perm[i]
        if constraint and not constraint(perm):
            perm[i], perm[j] = perm[j], perm[i]; continue
        c = cost(perm, U, B, sh)
        if c < cur or rnd.random() < math.exp((cur - c)/T): cur = c
        else: perm[i], perm[j] = perm[j], perm[i]
        if cur < best[0]: best = (cur, perm.copy())
    return best[1], best[0]

def board(s, perm):
    at = {int(p): u for u, p in zip(s["units"], perm)}
    rows = [range(0,10), range(10,19), range(19,26)]
    return "\n".join(("  "*r) + " ".join(at[i] for i in rr) for r, rr in enumerate(rows))

def stats(s, perm, U, B, sh, syl):
    hands = np.array([HAND(fin[p]) for p in perm]); fingers = np.array([fin[p] for p in perm])
    tot = U.sum()
    same_hand = sum(B[a,b] for a in range(N) for b in range(N) if hands[a]==hands[b]) / B.sum()
    sfb = sum(B[a,b] for a in range(N) for b in range(N) if a!=b and fingers[a]==fingers[b]) / B.sum()
    home = sum(U[u] for u in range(N) if POS[perm[u]][0]==1) / tot
    left = sum(U[u] for u in range(N) if hands[u]==0) / tot
    pinky = sum(U[u] for u in range(N) if fingers[u] in (0,7)) / tot
    return dict(keys_per_syl=round(tot/syl,3), effort_per_syl=round(cost(perm,U,B,sh)/syl,3),
                shift_pct=round(100*sh/tot,2), home_pct=round(100*home,1), left_pct=round(100*left,1),
                same_hand_pct=round(100*same_hand,1), sfb_pct=round(100*sfb,2), pinky_pct=round(100*pinky,1))

TWO_SET = make_system("2-set (KS X 5002)", "ㅏㅓㅗㅜㅡㅣㅕㅑㅛㅠㅐㅔ", "", False, shifted_vowels="ㅒㅖ")
TWO_SET_POS = {u: KEYS.index(k) for u, k in zip("ㅂㅈㄷㄱㅅㅛㅕㅑㅐㅔㅁㄴㅇㄹㅎㅗㅓㅏㅣㅋㅌㅊㅍㅠㅜㅡ", KEYS)}

if __name__ == "__main__":
    per = int(sys.argv[1]) if len(sys.argv) > 1 else 1_000_000
    sample = collections.Counter(w for _, w in words(per)).items()
    systems = [
        (TWO_SET, "fixed"),
        (TWO_SET, "free"),
        (make_system("2-set + implied ㅇ", "ㅏㅓㅗㅜㅡㅣㅕㅑㅛㅠㅐㅔ", "", True, "ㅒㅖ"), "free"),
        (make_system("bada A: ㅆ ㅛ ㅘ keyed, ㅑㅠ→ㅣ+, implied ㅇ", "ㅏㅓㅗㅜㅡㅣㅕㅐㅔ", "ㅆㅛㅘ", True), "free"),
        (make_system("bada B: ㅆ ㅛ ㄲ keyed", "ㅏㅓㅗㅜㅡㅣㅕㅐㅔ", "ㅆㅛㄲ", True), "free"),
        (make_system("bada C: ㅆ ㅛ ㅢ keyed", "ㅏㅓㅗㅜㅡㅣㅕㅐㅔ", "ㅆㅛㅢ", True), "free"),
        (make_system("bada D: ㅆ ㅑ ㅛ ㅠ keyed, ㅔ→ㅓㅣ", "ㅏㅓㅗㅜㅡㅣㅕㅐㅑㅛㅠ", "ㅆ", True), "free"),
    ]
    out = []
    for s, mode in systems:
        if "ㄲ" in s["keyed"]: TENSE.pop("ㄲ", None)
        uni, bi, syl, _ = stream(s, sample)
        if "ㄲ" in s["keyed"]: TENSE["ㄲ"] = "ㄱ"
        U, B, sh = matrices(s, uni, bi)
        if mode == "fixed":
            perm, c = anneal(U, B, sh, fixed=[TWO_SET_POS[u] for u in s["units"]])
        else:
            perm, c = min((anneal(U, B, sh, seed=k) for k in range(4)), key=lambda r: r[1])
        st = stats(s, perm, U, B, sh, syl)
        print(f"\n### {s['name']} [{mode}]\n{json.dumps(st, ensure_ascii=False)}\n{board(s, perm)}", flush=True)
        out.append(dict(name=s["name"], mode=mode, stats=st, board=board(s, perm)))
    json.dump(out, open("out_results.json", "w"), ensure_ascii=False, indent=1)
