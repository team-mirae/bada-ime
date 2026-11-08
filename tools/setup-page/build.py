"""Render a setup page from an HHKB Keymap Tool file (.hks) and a bada settings export.

Usage: python3 tools/setup-page/build.py KEYMAP.hks bada-settings.json OUT.html
"""
import json, os, sys
hks = json.load(open(sys.argv[1])); bada = json.load(open(sys.argv[2]))["settings"]
# HHKB Keymap Tool position → key (KeyboardEvent.code; FnL/FnR for the fn keys)
POS = [None,"ArrowRight","ArrowDown","ArrowLeft","FnR","AltRight","MetaRight","Lang1","Space","Lang2","MetaLeft","AltLeft","Backquote","FnL",
 "ShiftRight","ArrowUp","IntlRo","Slash","Period","Comma","KeyM","KeyN","KeyB","KeyV","KeyC","KeyX","KeyZ","ShiftLeft",
 "Backslash","Quote","Semicolon","KeyL","KeyK","KeyJ","KeyH","KeyG","KeyF","KeyD","KeyS","KeyA","ControlLeft",
 "Enter","BracketRight","BracketLeft","KeyP","KeyO","KeyI","KeyU","KeyY","KeyT","KeyR","KeyE","KeyW","KeyQ","Tab",
 "Backspace","IntlYen","Equal","Minus","Digit0","Digit9","Digit8","Digit7","Digit6","Digit5","Digit4","Digit3","Digit2","Digit1","Escape"]
layers = {}
for name, codes in hks["Keymap"].items():
    layers[name] = {POS[i]: c for i, c in enumerate(codes[:len(POS)]) if POS[i]}
# The renderer's layouts (tools/render/layouts.js) that the setup uses, for the legends.
src = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "render", "layouts.js"), encoding="utf-8").read()
all_layouts = json.loads(src[src.index("["):src.rindex("]") + 1])
used = {0, bada.get("default_muhenkan", 0), *bada.get("enabled", [])}
for p in bada.get("profiles", {}).values():
    used.update(p.get("enabled", []))
layouts = {l["id"]: {"name": l["display"], "keys": l["keys"]} for l in all_layouts if l["id"] in used}
data = {"hhkb": {k: v for k, v in hks.items() if k != "Keymap"}, "layers": layers, "bada": bada, "layouts": layouts}
page = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "template.html")).read().replace("/*DATA*/", json.dumps(data, ensure_ascii=False))
open(sys.argv[3], "w").write(page)
print("ok", len(page))
