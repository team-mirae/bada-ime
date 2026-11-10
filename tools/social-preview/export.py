"""Render the GitHub social preview (docs/images/social-preview.png, 1280×640) with headless Chrome.

    python3 tools/social-preview/export.py [--chrome PATH]

Upload it in the repository's Settings → General → Social preview.
"""
import argparse, os, subprocess, tempfile, urllib.parse

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.normpath(os.path.join(HERE, "../../docs/images/social-preview.png"))
CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--chrome", default=CHROME)
    a = ap.parse_args()
    page = "file://" + urllib.parse.quote(os.path.join(HERE, "index.html"))
    with tempfile.TemporaryDirectory() as profile:
        # headless Chrome's viewport is shorter than its window: shoot taller, then crop
        subprocess.run([a.chrome, "--headless=new", f"--user-data-dir={profile}", "--hide-scrollbars",
                        "--allow-file-access-from-files", "--force-device-scale-factor=1",
                        "--window-size=1280,800", f"--screenshot={OUT}", page],
                       check=True, capture_output=True)
    from PIL import Image   # sips crops around the centre; the page is the top 640 rows
    Image.open(OUT).crop((0, 0, 1280, 640)).save(OUT, optimize=True)
    print(OUT, os.path.getsize(OUT), "bytes")


if __name__ == "__main__":
    main()
