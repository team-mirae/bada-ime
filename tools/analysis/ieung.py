"""Where initial ㅇ appears, and how often an implied ㅇ would collide with a vowel compound."""
import collections, sys
from corpus import words, decompose
COMP = {("ㅗ","ㅏ"),("ㅗ","ㅐ"),("ㅗ","ㅣ"),("ㅜ","ㅓ"),("ㅜ","ㅔ"),("ㅜ","ㅣ"),("ㅡ","ㅣ")}  # 2-set compounds
EXT = {("ㅣ","ㅏ"),("ㅣ","ㅓ"),("ㅣ","ㅗ"),("ㅣ","ㅜ"),("ㅕ","ㅣ"),("ㅑ","ㅣ"),("ㅏ","ㅣ"),("ㅓ","ㅣ")}
c = collections.Counter(); syl = 0; ex = collections.Counter()
for src, w in words(int(sys.argv[1]) if len(sys.argv) > 1 else 2_000_000):
    prev = None
    for ch in w:
        l, v, t = decompose(ch); syl += 1
        if l == "ㅇ":
            if prev is None: c["word start"] += 1
            elif prev[2]: c["after final"] += 1
            else:
                c["after open"] += 1
                pv = prev[1][-1:]  # crude: last component
                if (prev[1], v) in COMP: c["  collides 2-set compound"] += 1; ex[prev[0]+prev[1]+"·"+v] += 1
                if (prev[1], v) in EXT: c["  collides extended compound"] += 1
        prev = (l, v, t)
for k, n in c.items(): print(f"{k:32s}{100*n/syl:6.2f}% of syllables")
print(ex.most_common(12))
