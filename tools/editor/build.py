"""Inline data.json into editor.src.html -> index.html (one self-contained page)."""
import os
here = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(here, "editor.src.html")).read()
data = open(os.path.join(here, "data.json")).read().replace("</", "<\\/")
open(os.path.join(here, "index.html"), "w").write(src.replace("/*DATA*/", data))
print("index.html", len(src) + len(data), "bytes")
