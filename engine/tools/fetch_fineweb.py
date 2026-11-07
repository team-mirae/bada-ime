"""Counts Korean words in part of FineWeb-2's Korean web text (ODC-By), for build_ko_words.py.

Reads the `text` column of every STEP-th row group of one shard over HTTP (range requests; nothing
else is downloaded), counts Hangul words (eojeol) and chat jamo, and writes
engine/tools/cache/fineweb-ko_words.tsv (counts ≥ 2). No text is kept.

FineWeb-2: HuggingFaceFW/fineweb-2, data/kor_Hang, ODC-By 1.0 (https://huggingface.co/datasets/HuggingFaceFW/fineweb-2).
Needs pyarrow and fsspec (tools/analysis/.venv). Usage: fetch_fineweb.py [SHARD] [STEP]
"""
import collections, os, re, sys, time
import fsspec, pyarrow.parquet as pq

SHARD = sys.argv[1] if len(sys.argv) > 1 else "000_00000"
STEP = int(sys.argv[2]) if len(sys.argv) > 2 else 4
URL = f"https://huggingface.co/datasets/HuggingFaceFW/fineweb-2/resolve/main/data/kor_Hang/train/{SHARD}.parquet"
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "cache", "fineweb-ko_words.tsv")
WORD = re.compile(r"[가-힣]+|[ㄱ-ㅣ]{2,}")

f = fsspec.open(URL, "rb", block_size=1 << 22).open()
pf = pq.ParquetFile(f)
groups = list(range(0, pf.metadata.num_row_groups, STEP))
counts = collections.Counter()
docs = 0
started = time.time()
for n, g in enumerate(groups):
    table = pf.read_row_group(g, columns=["text"])
    for text in table.column("text").to_pylist():
        counts.update(WORD.findall(text))
    docs += table.num_rows
    if n % 50 == 0:
        print(f"{n}/{len(groups)} groups, {docs} docs, {len(counts)} words, {time.time() - started:.0f}s", flush=True)
with open(OUT, "w") as out:
    for w, c in counts.most_common():
        if c < 2:
            break
        out.write(f"{w}\t{c}\n")
print(f"done: {docs} docs, {len(counts)} words, {time.time() - started:.0f}s; 키로깅 {counts.get('키로깅', 0)}, 초성 {counts.get('초성', 0)}")
