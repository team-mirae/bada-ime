"""Render the bada ime icon (docs/images/icon/bada-icon.svg) to PNGs beside it, with headless Chrome.

    python3 tools/icon/export.py [--chrome PATH]

The PNGs: bada-icon-1024.png and bada-icon-512.png (the Mac's Settings → About uses the 512). The
input menu's icon is not this one: it stays 바 in a square (mac/Resources/icons).
"""
import argparse, os, subprocess, tempfile, urllib.parse

HERE = os.path.dirname(os.path.abspath(__file__))
DIR = os.path.normpath(os.path.join(HERE, "../../docs/images/icon"))
CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--chrome", default=CHROME)
    a = ap.parse_args()
    from PIL import Image
    svg = "file://" + urllib.parse.quote(os.path.join(DIR, "bada-icon.svg"))
    with tempfile.TemporaryDirectory() as tmp:
        page = os.path.join(tmp, "icon.html")
        open(page, "w").write(f'<html><body style="margin:0"><img src="{svg}" width="1024" height="1024"></body></html>')
        raw = os.path.join(tmp, "raw.png")
        # headless Chrome's viewport is shorter than its window: shoot taller, then crop
        subprocess.run([a.chrome, "--headless=new", f"--user-data-dir={tmp}/profile", "--hide-scrollbars",
                        "--allow-file-access-from-files", "--default-background-color=00000000",
                        "--force-device-scale-factor=1", "--window-size=1024,1200", f"--screenshot={raw}",
                        "file://" + page], check=True, capture_output=True)
        icon = Image.open(raw).crop((0, 0, 1024, 1024))
        for size in (1024, 512):
            out = os.path.join(DIR, f"bada-icon-{size}.png")
            (icon if size == 1024 else icon.resize((size, size), Image.LANCZOS)).save(out, optimize=True)
            print(out, os.path.getsize(out), "bytes")


if __name__ == "__main__":
    main()
