"""engine/data/ko-jamo-4gram.bin: a jamo 4-gram language model for typo suggestions (engine/src/typo.rs).

Trained on engine/data/ko-words.tsv (each word weighted by its count). A word is its jamo in
conjoining form (initial, vowel and final are different symbols), between three start marks and an
end mark; stray compatibility jamo (ㅋㅋ) are symbols of their own. Every n-gram of order 1–4 is kept
with its count, and every history with its total count and number of distinct next symbols, for
Witten–Bell interpolation at run time.

Token ids (must match typo.rs): 1 start, 2 end, 3 other, 4–22 initials, 23–43 vowels, 44–70 finals,
71–121 compatibility jamo U+3131–U+3163. An n-gram's key is its ids folded 7 bits each, oldest first.

File (little-endian): b"BJLM", u32 version 1, u32 n, n × (u32 key, u32 count),
u32 m, m × (u32 history key, u32 total, u32 distinct); both sorted by key.
Usage: python3 build_jamo_lm.py
"""
import collections, os, struct, sys

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "../data/ko-words.tsv")
OUT = os.path.join(HERE, "../data/ko-jamo-4gram.bin")
START, END, OTHER = 1, 2, 3


def ids(word):
    out = []
    for ch in word:
        o = ord(ch)
        if 0xAC00 <= o <= 0xD7A3:
            n = o - 0xAC00
            out.append(4 + n // 588)
            out.append(23 + (n % 588) // 28)
            if n % 28:
                out.append(43 + n % 28)
        elif 0x3131 <= o <= 0x3163:
            out.append(71 + o - 0x3131)
        else:
            out.append(OTHER)
    return out


def key(seq):
    k = 0
    for t in seq:
        k = (k << 7) | t
    return k


grams = collections.Counter()
for line in open(SRC, encoding="utf-8"):
    if line.startswith("#"):
        continue
    word, count = line.rstrip("\n").split("\t")
    count = int(count)
    toks = [START] * 3 + ids(word) + [END]
    for i in range(3, len(toks)):
        for n in range(1, 5):
            grams[tuple(toks[i - n + 1:i + 1])] += count

total = collections.Counter()
distinct = collections.Counter()
for g, c in grams.items():
    h = key(g[:-1])
    total[h] += c
    distinct[h] += 1

cap = lambda c: min(c, 0xFFFFFFFF)
with open(OUT, "wb") as f:
    f.write(b"BJLM" + struct.pack("<I", 1))
    rows = sorted((key(g), cap(c)) for g, c in grams.items())
    f.write(struct.pack("<I", len(rows)))
    for k, c in rows:
        f.write(struct.pack("<II", k, c))
    hist = sorted((h, cap(total[h]), distinct[h]) for h in total)
    f.write(struct.pack("<I", len(hist)))
    for h, t, d in hist:
        f.write(struct.pack("<III", h, t, d))
print(f"{len(grams)} n-grams, {len(total)} histories, {os.path.getsize(OUT) // 1024} KiB", file=sys.stderr)
