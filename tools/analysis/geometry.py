"""The 48 JIS kana-space keys: canonical code, row, column, finger, effort.

Rows: 0 number, 1 Q, 2 A, 3 Z. Fingers 0 L pinky .. 7 R pinky. Effort is a generic per-key cost
(home row index = 0.7). Only these keys carry Korean in Bada; Shift on the 22 non-letter keys is
reserved for digits and symbols.
"""
ROWS = [
    ["Digit1","Digit2","Digit3","Digit4","Digit5","Digit6","Digit7","Digit8","Digit9","Digit0","Minus","Equal","IntlYen"],
    ["KeyQ","KeyW","KeyE","KeyR","KeyT","KeyY","KeyU","KeyI","KeyO","KeyP","BracketLeft","BracketRight"],
    ["KeyA","KeyS","KeyD","KeyF","KeyG","KeyH","KeyJ","KeyK","KeyL","Semicolon","Quote","Backslash"],
    ["KeyZ","KeyX","KeyC","KeyV","KeyB","KeyN","KeyM","Comma","Period","Slash","IntlRo"],
]
EFFORT = [
    [3.0,2.6,2.4,2.4,2.8,3.0,2.4,2.4,2.6,3.0,3.4,3.8,4.2],
    [2.0,1.5,1.2,1.3,1.8,2.0,1.3,1.2,1.5,2.0,2.6,3.2],
    [1.3,1.0,0.8,0.7,1.4,1.4,0.7,0.8,1.0,1.3,2.0,2.6],
    [2.2,1.9,1.5,1.4,1.8,1.5,1.4,1.5,1.9,2.2,2.8],
]
COL_FINGER = [0,1,2,3,3,4,4,5,6,7,7,7,7]
KEYS = []          # (code, row, col, finger, effort)
for r, row in enumerate(ROWS):
    for c, code in enumerate(row):
        KEYS.append((code, r, c, COL_FINGER[c], EFFORT[r][c]))
LETTER = {k[0] for k in KEYS if k[0].startswith("Key")}
