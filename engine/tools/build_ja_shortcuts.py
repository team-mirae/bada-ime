"""engine/data/ja-shortcuts.tsv: Japanese romaji readings that type a Korean syllable in fewer keys,
for layout 15 (docs/layouts/15-bada-roman.md, "Accept Japanese romaji readings as shortcuts").

A character in both Korea's education hanja (kKoreanEducationHanja) and Japan's jōyō kanji
(kJoyoKanji; the Korean form matched to the jōyō form by kJapaneseNewVariant, 國 = 国) gives a
shortcut for each pair of its Korean reading (kHangul) and on'yomi (kJapaneseOn) where:

- the two sound alike: similarity 0.85 or more (`similarity` below: the initial, the vowel and the
  ending, by the regular Sino-Korean / Sino-Japanese correspondences);
- the Korean reading takes 4 or more keys in layout 18's default spelling, or with ㅕ as `yu`
  before a final (layout 15's interlaced vowels: nyun 년);
- the romaji takes fewer keys than that.

Rows: romaji, Korean syllable, the kanji (jōyō forms) giving it; most frequent Korean syllable first
for each romaji (counts from ko-words.tsv).

Sources: the Unicode Unihan database (Unicode License v3, engine/data/LICENSE-unicode.txt), in
engine/tools/cache/unicode/Unihan/ (not committed):

  curl -LO https://www.unicode.org/Public/UCD/latest/ucd/Unihan.zip && unzip Unihan.zip -d Unihan

Usage: python3 build_ja_shortcuts.py
"""
import json, os, re

HERE = os.path.dirname(os.path.abspath(__file__))
UNIHAN = os.path.join(HERE, "cache", "unicode", "Unihan")
DATA = os.path.join(HERE, "..", "data")
DOCS = os.path.join(HERE, "..", "..", "docs", "layouts")


def fields(name, keys):
    out = {k: {} for k in keys}
    for line in open(os.path.join(UNIHAN, name), encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        cp, key, val = line.rstrip("\n").split("\t", 2)
        if key in out:
            out[key][chr(int(cp[2:], 16))] = val
    return out


other = fields("Unihan_OtherMappings.txt", ["kJoyoKanji", "kKoreanEducationHanja"])
reads = fields("Unihan_Readings.txt", ["kHangul", "kJapaneseOn"])
variants = fields("Unihan_Variants.txt", ["kJapaneseNewVariant"])["kJapaneseNewVariant"]

joyo = {c for c, v in other["kJoyoKanji"].items() if not v.startswith("U+")}
edu = set(other["kKoreanEducationHanja"])
old_forms = {}  # jōyō form -> its traditional forms
for old, v in variants.items():
    for u in v.split():
        old_forms.setdefault(chr(int(u.split("<")[0][2:], 16)), []).append(old)

# Korean syllable counts, to order the syllables a romaji gives
syllable_count = {}
for line in open(os.path.join(DATA, "ko-words.tsv"), encoding="utf-8"):
    if line.startswith("#"):
        continue
    word, n = line.rstrip("\n").split("\t")
    for ch in word:
        if "가" <= ch <= "힣":
            syllable_count[ch] = syllable_count.get(ch, 0) + int(n)

spelling = json.load(open(os.path.join(DOCS, "18-korean-roman.json"), encoding="utf-8"))["spelling"]
ON = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ"
VO = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ"
CO = [""] + list("ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ")


def jamo(s):
    c = ord(s) - 0xAC00
    return ON[c // 588], VO[c % 588 // 28], CO[c % 28]


def romanize(s):
    o, v, f = jamo(s)
    return spelling["onsets"][o] + spelling["vowels"][v] + (spelling["codas"].get(f, "?") if f else "")


def keys(s):
    """Keys for the syllable in layout 15: the default spelling, or ㅕ as `yu` before a final (nyun 년)."""
    o, v, f = jamo(s)
    n = len(romanize(s))
    return n - 1 if v == "ㅕ" and f else n


# Sino-Korean / Sino-Japanese correspondences: 1 the regular one, 0.7 a common alternative, 0.5 weak
A, W = 0.7, 0.5
INITIAL = {
    "ㄱ": {"k": 1, "g": 1}, "ㄲ": {"k": 1, "g": 1}, "ㅋ": {"k": 1, "g": 1}, "ㅎ": {"k": 1, "g": 1, "h": A, "": A},
    "ㄷ": {"t": 1, "d": 1, "ch": A, "ts": A, "j": A}, "ㄸ": {"t": 1, "d": 1}, "ㅌ": {"t": 1, "d": 1, "ch": A, "ts": A},
    "ㅂ": {"h": 1, "b": 1, "f": 1, "p": A}, "ㅃ": {"h": 1, "b": 1}, "ㅍ": {"h": 1, "b": 1, "f": 1, "p": A},
    "ㅁ": {"m": 1, "b": 1}, "ㄴ": {"n": 1, "d": 1, "j": A, "z": A}, "ㄹ": {"r": 1},
    "ㅅ": {"s": 1, "sh": 1, "z": 1, "j": 1}, "ㅆ": {"s": 1, "sh": 1},
    "ㅈ": {"s": 1, "sh": 1, "z": 1, "j": 1, "ch": 1, "ts": 1, "t": A, "d": A}, "ㅉ": {"s": 1, "sh": 1, "ch": 1},
    "ㅊ": {"s": 1, "sh": 1, "z": 1, "j": 1, "ch": 1, "ts": 1, "t": A, "d": A},
    "ㅇ": {"": 1, "y": 1, "w": 1, "g": A, "n": W, "j": W, "z": W},
}
VOWEL = {
    "ㅏ": {"a": 1, "o": W}, "ㅑ": {"a": 1}, "ㅓ": {"o": 1, "e": A}, "ㅕ": {"o": 1, "e": 1}, "ㅗ": {"o": 1, "u": A},
    "ㅛ": {"o": 1, "u": 1}, "ㅜ": {"u": 1, "o": A}, "ㅠ": {"u": 1, "o": A}, "ㅡ": {"u": .8, "i": .8, "o": A},
    "ㅣ": {"i": 1, "e": A}, "ㅐ": {"a": 1, "e": A}, "ㅔ": {"e": 1, "a": A}, "ㅖ": {"e": 1, "a": A}, "ㅒ": {"a": A},
    "ㅚ": {"a": .8, "o": A}, "ㅟ": {"i": 1}, "ㅢ": {"i": 1, "u": A}, "ㅘ": {"a": 1}, "ㅝ": {"e": .8, "o": .8},
    "ㅙ": {"a": A}, "ㅞ": {"e": A},
}
ENDING = {
    "": {"": 1, "i": A, "u": A}, "ㄱ": {"ku": 1, "ki": 1}, "ㄴ": {"n": 1}, "ㅁ": {"n": 1},
    "ㄹ": {"tsu": 1, "chi": 1}, "ㅂ": {"u": .9, "tsu": A}, "ㅇ": {"u": 1, "i": 1, "": W},
}


def similarity(syllable, romaji):
    m = re.match(r"^([^aeiou]*?)(y|w)?([aeiou])(.*)$", romaji)
    if not m:
        return 0
    o, v, f = jamo(syllable)
    return (.35 * INITIAL.get(o, {}).get(m.group(1), 0) + .35 * VOWEL.get(v, {}).get(m.group(3), 0)
            + .3 * ENDING.get(f, {}).get(m.group(4), 0))


pairs = {}  # (romaji, syllable) -> kanji
for k in sorted(joyo):
    korean = k if k in edu else next((o for o in old_forms.get(k, []) if o in edu), None)
    if not korean or k not in reads["kJapaneseOn"]:
        continue
    hangul = [h.split(":") for h in reads["kHangul"].get(korean, "").split()]
    syllables = [h for h, tag in hangul if "E" in tag] or [h for h, _ in hangul]
    for s in dict.fromkeys(syllables):
        n = keys(s)
        for on in reads["kJapaneseOn"][k].lower().split():
            if n >= 4 and len(on) < n and similarity(s, on) >= 0.85:
                pairs.setdefault((on, s), []).append(k)

rows = sorted(pairs.items(), key=lambda p: (p[0][0], -syllable_count.get(p[0][1], 0)))
with open(os.path.join(DATA, "ja-shortcuts.tsv"), "w", encoding="utf-8") as out:
    out.write("# Japanese romaji readings that type a Korean syllable in fewer keys (layout 15). Generated by\n"
              "# tools/build_ja_shortcuts.py from Unicode Unihan (LICENSE-unicode.txt).\n"
              "# romaji<TAB>Korean syllable<TAB>kanji; per romaji, the most frequent syllable first.\n")
    for (on, s), ks in rows:
        out.write(f"{on}\t{s}\t{''.join(ks)}\n")
print(f"{len(rows)} rows, {len({on for on, _ in pairs})} romaji")
