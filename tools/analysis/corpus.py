"""Shared corpus loader: a balanced Korean sample across registers.

Reads JIS Moonshot's gitignored corpora (set BADA_CORPORA, default ~/Projects/jis-moonshot/corpora).
Each register contributes the same number of Hangul syllables, so news doesn't drown chat.
"""
import os, re, json, random

ROOT = os.path.expanduser(os.environ.get("BADA_CORPORA", "~/Projects/jis-moonshot/corpora"))
SOURCES = {
    "news":   "derived/leipzig-corpora/ko.txt",
    "formal": "derived/korean-petitions/ko.txt",
    "review": "nsmc-ratings.txt",
    "chat":   "derived/korean-safe-conversation/ko.txt",
}
HANGUL = re.compile(r"[가-힣]+")
L = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ"
V = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ"
T = ["", "ㄱ","ㄲ","ㄳ","ㄴ","ㄵ","ㄶ","ㄷ","ㄹ","ㄺ","ㄻ","ㄼ","ㄽ","ㄾ","ㄿ","ㅀ","ㅁ","ㅂ","ㅄ","ㅅ","ㅆ","ㅇ","ㅈ","ㅊ","ㅋ","ㅌ","ㅍ","ㅎ"]

def decompose(ch):
    n = ord(ch) - 0xAC00
    return L[n // 588], V[(n % 588) // 28], T[n % 28]

def words(per_source=3_000_000, seed=1):
    """Yield (source, word) Hangul runs; ~per_source syllables from each source."""
    rnd = random.Random(seed)
    for name, rel in SOURCES.items():
        path = os.path.join(ROOT, rel)
        n = 0
        with open(path, encoding="utf-8", errors="ignore") as f:
            for line in f:
                for w in HANGUL.findall(line):
                    yield name, w
                    n += len(w)
                if n >= per_source:
                    break
