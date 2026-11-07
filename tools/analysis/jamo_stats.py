"""Jamo frequencies by position, and what standard 2-set costs (keys, Shift)."""
import collections, sys
from corpus import words, decompose

C = collections.Counter
ini, med, fin, src_syl = C(), C(), C(), C()
syl = 0
for src, w in words(int(sys.argv[1]) if len(sys.argv) > 1 else 3_000_000):
    for ch in w:
        l, v, t = decompose(ch)
        ini[l] += 1; med[v] += 1; fin[t or "∅"] += 1; syl += 1; src_syl[src] += 1

def show(title, c):
    print(f"\n## {title}")
    for k, n in c.most_common():
        print(f"{k}\t{100*n/syl:6.2f}%")
print("syllables", syl, dict(src_syl))
show("initial", ini); show("medial", med); show("final", fin)

# standard 2-set keystrokes
SHIFT = set("ㅃㅉㄸㄲㅆㅒㅖ")
VSPLIT = {"ㅘ":"ㅗㅏ","ㅙ":"ㅗㅐ","ㅚ":"ㅗㅣ","ㅝ":"ㅜㅓ","ㅞ":"ㅜㅔ","ㅟ":"ㅜㅣ","ㅢ":"ㅡㅣ"}
TSPLIT = {"ㄳ":"ㄱㅅ","ㄵ":"ㄴㅈ","ㄶ":"ㄴㅎ","ㄺ":"ㄹㄱ","ㄻ":"ㄹㅁ","ㄼ":"ㄹㅂ","ㄽ":"ㄹㅅ","ㄾ":"ㄹㅌ","ㄿ":"ㄹㅍ","ㅀ":"ㄹㅎ","ㅄ":"ㅂㅅ"}
keys = C()
for k, n in ini.items(): keys[k] += n
for k, n in med.items():
    for j in VSPLIT.get(k, k): keys[j] += n
for k, n in fin.items():
    if k != "∅":
        for j in TSPLIT.get(k, k): keys[j] += n
total = sum(keys.values()); sh = sum(n for k, n in keys.items() if k in SHIFT)
print(f"\n## 2-set: {total/syl:.3f} keys/syllable, Shift on {100*sh/total:.2f}% of keys")
for k in sorted(SHIFT, key=lambda k: -keys[k]):
    print(f"{k}\t{100*keys[k]/total:5.2f}% of keys")
