"""Summarise bada ime's 초성 diagnostics (spec §8): for each initial key, the syllables it became.

Usage: python3 choseong_report.py [file] [--layout 13] [--top 8]
Default file: ~/Library/Application Support/bada-ime/choseong-diagnostics.json
Counts only: the file holds no words or text.
"""
import argparse, collections, json, os

ap = argparse.ArgumentParser()
ap.add_argument("file", nargs="?", default=os.path.expanduser("~/Library/Application Support/bada-ime/choseong-diagnostics.json"))
ap.add_argument("--layout", type=int, default=None)
ap.add_argument("--top", type=int, default=8)
args = ap.parse_args()

V = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ"
I = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ"
F = ["", "ㄱ","ㄲ","ㄳ","ㄴ","ㄵ","ㄶ","ㄷ","ㄹ","ㄺ","ㄻ","ㄼ","ㄽ","ㄾ","ㄿ","ㅀ","ㅁ","ㅂ","ㅄ","ㅅ","ㅆ","ㅇ","ㅈ","ㅊ","ㅋ","ㅌ","ㅍ","ㅎ"]
syl = lambda i, v, f: chr(0xAC00 + (I.index(i) * 21 + V.index(v)) * 28 + F.index(f)) if i in I and v in V and f in F else i + v + f

rows = json.load(open(args.file)).get("rows", [])
if args.layout is not None:
    rows = [r for r in rows if r["layout"] == args.layout]
total = sum(r["count"] for r in rows)
print(f"{args.file}\n{len(rows)} kinds, {total} syllables" + (f" (layout {args.layout})" if args.layout is not None else ""))

for source in ("typed", "choseong"):
    by_key = collections.defaultdict(collections.Counter)
    for r in rows:
        if r["source"] == source:
            k = ("⇧" if r["shift"] else "") + r["key"] + f" [{r['layout']}]"
            by_key[(r["initial"], k)][syl(r["initial"], r["vowel"], r["final_"])] += r["count"]
    if not by_key:
        continue
    print(f"\n== {source}: what each initial key became ==")
    for (ini, key), c in sorted(by_key.items(), key=lambda x: (I.index(x[0][0]) if x[0][0] in I else 99, x[0][1])):
        n = sum(c.values())
        top = "  ".join(f"{s} {100*v/n:.0f}%" for s, v in c.most_common(args.top))
        print(f"{ini} {key:<18} {n:>6}  {top}")

# The same initial on two keys: how differently they're used.
print("\n== one jamo, several keys (typed + choseong) ==")
by_ini = collections.defaultdict(lambda: collections.defaultdict(collections.Counter))
for r in rows:
    by_ini[(r["layout"], r["initial"])][("⇧" if r["shift"] else "") + r["key"]][r["vowel"]] += r["count"]
for (layout, ini), keys in sorted(by_ini.items()):
    if len(keys) < 2:
        continue
    print(f"{ini} [{layout}]")
    for key, c in sorted(keys.items(), key=lambda x: -sum(x[1].values())):
        n = sum(c.values())
        print(f"   {key:<14} {n:>6}  " + "  ".join(f"{v} {100*k/n:.0f}%" for v, k in c.most_common(6)))
