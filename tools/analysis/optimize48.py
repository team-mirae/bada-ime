"""Bada on the full JIS kana space: 48 unshifted keys + Shift on the 26 letter keys.

Compares a 2-set-style system (one key per consonant, initial or final) with a 3-set-style one
(separate initial and final keys), each placed by simulated annealing.
"""
import collections, json, math, random, sys
import numpy as np
from corpus import words, decompose
from geometry import KEYS, LETTER

SHIFT_COST = 2.0
NK = len(KEYS)
# slots: 0..47 unshifted keys, 48.. shifted letter keys
SLOTS = [(i, False) for i in range(NK)] + [(i, True) for i, k in enumerate(KEYS) if k[0] in LETTER]
NS = len(SLOTS)
eff = np.array([KEYS[k][4] + (SHIFT_COST if sh else 0) for k, sh in SLOTS])
key_of = np.array([k for k, _ in SLOTS])
P = np.zeros((NK, NK))
for i, (_, ri, ci, fi, _) in enumerate(KEYS):
    for j, (_, rj, cj, fj, _) in enumerate(KEYS):
        hi, hj = fi >= 4, fj >= 4
        if i == j: P[i, j] = 0.3
        elif fi == fj: P[i, j] = 2.5 + 1.0 * (abs(ri - rj) - 1 if abs(ri - rj) > 1 else 0) + (0.5 if ci != cj and abs(ri-rj)==0 else 0)
        elif hi == hj:
            P[i, j] = 0.3 + 0.4 * max(0, abs(ri - rj) - 1)
            inward = (fj > fi) if not hi else (fj < fi)
            if inward and abs(fi - fj) == 1 and ri == rj: P[i, j] -= 0.2
PS = P[np.ix_(key_of, key_of)]   # slot x slot

CLUSTER = {"ㄳ":"ㄱㅅ","ㄵ":"ㄴㅈ","ㄶ":"ㄴㅎ","ㄺ":"ㄹㄱ","ㄻ":"ㄹㅁ","ㄼ":"ㄹㅂ","ㄽ":"ㄹㅅ","ㄾ":"ㄹㅌ","ㄿ":"ㄹㅍ","ㅀ":"ㄹㅎ","ㅄ":"ㅂㅅ","ㄲ":"ㄱㄱ","ㅋ":"ㄱㅎ"}  # ㅋ final (0.00%) only stands in when unkeyed
VSEQ = {"ㅘ":"ㅗㅏ","ㅙ":"ㅗㅐ","ㅚ":"ㅗㅣ","ㅝ":"ㅜㅓ","ㅞ":"ㅜㅔ","ㅟ":"ㅜㅣ","ㅢ":"ㅡㅣ",
        "ㅑ":"ㅣㅏ","ㅕ":"ㅣㅓ","ㅛ":"ㅣㅗ","ㅠ":"ㅣㅜ","ㅒ":"ㅑㅣ","ㅖ":"ㅕㅣ","ㅐ":"ㅏㅣ","ㅔ":"ㅓㅣ"}
INITIALS = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ"

class System:
    def __init__(self, name, kind, vowels, finals=""):
        self.name, self.kind = name, kind          # kind: "2" (shared consonants) or "3"
        self.vowels = set(vowels); self.finals = set(finals)
        if kind == "3":
            self.units = ["i"+c for c in INITIALS] + ["v"+v for v in vowels] + ["f"+f for f in finals]
        else:
            self.units = ["c"+c for c in INITIALS] + ["v"+v for v in vowels]
        self.comp = set()
        for v in VSEQ:
            seq = self.vowel(v)
            for a, b in zip(seq, seq[1:]): self.comp.add((a, b))
    def vowel(self, v):
        if v in self.vowels: return ["v"+v]
        return [u for p in VSEQ[v] for u in self.vowel(p)]
    def final(self, t):
        p = "f" if self.kind == "3" else "c"
        have = self.finals if self.kind == "3" else set(INITIALS)
        if t in have: return [p+t]
        return [u for x in CLUSTER[t] for u in self.final(x)]
    def initial(self, l):
        return [("i" if self.kind == "3" else "c") + l]
    def type_word(self, w):
        keys = []; prev = None
        for ch in w:
            l, v, t = decompose(ch); vk = self.vowel(v)
            implied = False
            if l == "ㅇ":
                if self.kind == "3":   # a vowel after a final, or after a vowel it can't join, starts a syllable
                    implied = prev is None or prev[2] or (prev[3][-1], vk[0]) not in self.comp
                else:                  # 2-set: only at word start or after an open syllable
                    implied = prev is None or (not prev[2] and (prev[3][-1], vk[0]) not in self.comp)
            if not implied: keys += self.initial(l)
            keys += vk
            if t: keys += self.final(t)
            prev = (l, v, t, vk)
        return keys

def counts(s, sample):
    idx = {u: i for i, u in enumerate(s.units)}; n = len(s.units)
    U = np.zeros(n); B = np.zeros((n, n)); syl = 0
    for w, c in sample:
        ks = [idx[k] for k in s.type_word(w)]; syl += c * len(w)
        for k in ks: U[k] += c
        for a, b in zip(ks, ks[1:]): B[a, b] += c
    return U, B, syl

def cost(assign, U, B):
    return U @ eff[assign] + (B * PS[np.ix_(assign, assign)]).sum()

def anneal(U, B, iters=150000, seed=0):
    rnd = random.Random(seed); n = len(U)
    slots = list(range(NS)); rnd.shuffle(slots)
    assign = np.array(slots[:n]); free = slots[n:]
    cur = cost(assign, U, B); best = (cur, assign.copy()); T0 = cur * 0.02
    for it in range(iters):
        T = T0 * (1 - it / iters) ** 2 + 1e-9
        a = rnd.randrange(n)
        if free and rnd.random() < 0.3:          # move a unit to a free slot
            fi = rnd.randrange(len(free)); old = assign[a]; assign[a] = free[fi]
            c = cost(assign, U, B)
            if c < cur or rnd.random() < math.exp((cur - c) / T): cur = c; free[fi] = old
            else: assign[a] = old
        else:
            b = rnd.randrange(n)
            if a == b: continue
            assign[a], assign[b] = assign[b], assign[a]
            c = cost(assign, U, B)
            if c < cur or rnd.random() < math.exp((cur - c) / T): cur = c
            else: assign[a], assign[b] = assign[b], assign[a]
        if cur < best[0]: best = (cur, assign.copy())
    return best[1], best[0]

def report(s, assign, U, B, syl):
    tot = U.sum(); keys = key_of[assign]
    fing = np.array([KEYS[k][3] for k in keys]); hand = fing >= 4; row = np.array([KEYS[k][1] for k in keys])
    shifted = np.array([SLOTS[a][1] for a in assign])
    sfb = sum(B[i, j] for i in range(len(U)) for j in range(len(U)) if keys[i] != keys[j] and fing[i] == fing[j]) / B.sum()
    same = sum(B[i, j] for i in range(len(U)) for j in range(len(U)) if hand[i] == hand[j]) / B.sum()
    return dict(keys_per_syl=round(tot/syl, 3), effort_per_syl=round(cost(assign, U, B)/syl, 3),
                shift_pct=round(100*U[shifted].sum()/tot, 2), number_row_pct=round(100*U[row == 0].sum()/tot, 1),
                home_pct=round(100*U[row == 2].sum()/tot, 1), left_pct=round(100*U[~hand].sum()/tot, 1),
                same_hand_pct=round(100*same, 1), sfb_pct=round(100*sfb, 2),
                pinky_pct=round(100*U[(fing == 0) | (fing == 7)].sum()/tot, 1))

def layout_json(s, assign, U):
    out = {}
    for u, a, n in zip(s.units, assign, U):
        k, sh = SLOTS[a]; code = KEYS[k][0]
        out.setdefault(code, {})["shift" if sh else "base"] = {"t": u[1:], "c": u[0], "f": round(float(n), 1)}
    return out

SYSTEMS = [
    System("2-set, all vowels keyed, tense keyed, implied ㅇ", "2", "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ"),
    System("3-set A: 12 vowels + ㅘ, 16 finals", "3", "ㅏㅐㅑㅓㅔㅕㅗㅘㅛㅜㅠㅡㅣ", "ㄱㄴㄷㄹㅁㅂㅅㅆㅇㅈㅊㅌㅍㅎㄶㅄ"),
    System("3-set B: + ㅢ ㅚ ㅝ ㅟ ㅖ, all 27 finals", "3", "ㅏㅐㅑㅓㅔㅕㅖㅗㅘㅚㅛㅜㅝㅟㅠㅡㅢㅣ", "ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ"),
    System("3-set C: + ㅢ ㅚ, 18 finals", "3", "ㅏㅐㅑㅓㅔㅕㅗㅘㅚㅛㅜㅠㅡㅢㅣ", "ㄱㄲㄴㄷㄹㅁㅂㅅㅆㅇㅈㅊㅋㅌㅍㅎㄶㅄ"),
]

if __name__ == "__main__":
    per = int(sys.argv[1]) if len(sys.argv) > 1 else 1_000_000
    sample = list(collections.Counter(w for _, w in words(per)).items())
    results = []
    for s in SYSTEMS:
        U, B, syl = counts(s, sample)
        assign, c = min((anneal(U, B, seed=k) for k in range(3)), key=lambda r: r[1])
        st = report(s, assign, U, B, syl)
        print(f"\n### {s.name} ({len(s.units)} units)\n{json.dumps(st, ensure_ascii=False)}", flush=True)
        results.append(dict(name=s.name, stats=st, layout=layout_json(s, assign, U)))
    json.dump(results, open("out_48.json", "w"), ensure_ascii=False, indent=1)
