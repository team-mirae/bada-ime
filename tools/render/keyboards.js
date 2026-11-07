// Physical keyboards, in key units (1u = one letter key). Each key: [code, x, y, w, h, label].
// code is the KeyboardEvent.code the layout keys on; null for keys macOS never sees (fn).
// "ENTER" is the ISO-shaped Return (1.5u on the Q row, 1.25u on the A row).
(function () {
  const row = (y, x0, codes) => { let x = x0; return codes.map(([c, w = 1, label]) => { const k = [c, x, y, w, 1, label]; x += w; return k; }); };
  const NUM = ["Digit1","Digit2","Digit3","Digit4","Digit5","Digit6","Digit7","Digit8","Digit9","Digit0","Minus","Equal","IntlYen"].map(c => [c]);
  const QROW = ["KeyQ","KeyW","KeyE","KeyR","KeyT","KeyY","KeyU","KeyI","KeyO","KeyP","BracketLeft","BracketRight"].map(c => [c]);
  const AROW = ["KeyA","KeyS","KeyD","KeyF","KeyG","KeyH","KeyJ","KeyK","KeyL","Semicolon","Quote","Backslash"].map(c => [c]);
  const ZROW = ["KeyZ","KeyX","KeyC","KeyV","KeyB","KeyN","KeyM","Comma","Period","Slash","IntlRo"].map(c => [c]);

  // HHKB Professional JIS, 15u x 5u (from the owner's HHKB Keycap Swap and HHKB JIS Kana artifacts).
  const hhkb = [
    ...row(0, 0, [["Escape", 1, "Esc"], ...NUM, ["Backspace", 1, "Delete"]]),
    ...row(1, 0, [["Tab", 1.5, "Tab"], ...QROW]),
    ...row(2, 0, [["ControlLeft", 1.75, "Control"], ...AROW]),
    ...row(3, 0, [["ShiftLeft", 2, "Shift"], ...ZROW, ["ArrowUp", 1, "↑"], ["ShiftRight", 1, "Shift"]]),
    ...row(4, 0, [[null, 1, "fn"], ["GAP", 0.25], ["Backquote", 1, "半角/全角"], ["AltLeft", 1, "⌥"], ["MetaLeft", 1, "⌘"],
                  ["Lang2", 1, "英数"], ["Space", 2.5, ""], ["Lang1", 1, "かな"], ["MetaRight", 1, "⌘"], ["AltRight", 1, "⌥"],
                  [null, 1, "fn"], ["GAP", 0.25], ["ArrowLeft", 1, "←"], ["ArrowDown", 1, "↓"], ["ArrowRight", 1, "→"]]),
    ["Enter", 13.5, 1, 1.5, 2, "Return", "ENTER"],
  ];

  // Generic Apple-style JIS: 半角/全角 left of 1, Control left of A, Caps Lock bottom left.
  // ↑ and ↓ share one 1u column at half height, as on Apple boards.
  const apple = [
    ...row(0, 0, [["Backquote", 1, "半角/全角"], ...NUM, ["Backspace", 1, "delete"]]),
    ...row(1, 0, [["Tab", 1.5, "tab"], ...QROW]),
    ...row(2, 0, [["ControlLeft", 1.75, "control"], ...AROW]),
    ...row(3, 0, [["ShiftLeft", 2.25, "shift"], ...ZROW, ["ShiftRight", 1.75, "shift"]]),
    ...row(4, 0, [["CapsLock", 1, "caps"], ["AltLeft", 1, "⌥"], ["MetaLeft", 1, "⌘"], ["Lang2", 1, "英数"], ["Space", 4, ""],
                  ["Lang1", 1, "かな"], ["MetaRight", 1, "⌘"], ["AltRight", 1, "⌥"], [null, 1, "🌐"], ["ArrowLeft", 1, "←"]]),
    ["ArrowUp", 13, 4, 1, 0.5, "↑"], ["ArrowDown", 13, 4.5, 1, 0.5, "↓"], ["ArrowRight", 14, 4, 1, 1, "→"],
    ["Enter", 13.5, 1, 1.5, 2, "return", "ENTER"],
  ];

  const clean = ks => ks.filter(k => k[0] !== "GAP");
  window.BADA_KEYBOARDS = {
    hhkb: { name: "HHKB JIS", width: 15, height: 5, keys: clean(hhkb) },
    apple: { name: "Apple-style JIS", width: 15, height: 5, keys: clean(apple) },
  };

  // JIS legends of the 48 kana-space keys: [unshifted, shifted]
  window.BADA_JIS = {
    Digit1:["1","!"],Digit2:["2","\""],Digit3:["3","#"],Digit4:["4","$"],Digit5:["5","%"],Digit6:["6","&"],Digit7:["7","'"],
    Digit8:["8","("],Digit9:["9",")"],Digit0:["0",""],Minus:["-","="],Equal:["^","~"],IntlYen:["¥","|"],
    BracketLeft:["@","`"],BracketRight:["[","{"],Semicolon:[";","+"],Quote:[":","*"],Backslash:["]","}"],
    Comma:[",","<"],Period:[".",">"],Slash:["/","?"],IntlRo:["\\","_"],
  };
  for (const c of "QWERTYUIOPASDFGHJKLZXCVBNM") window.BADA_JIS["Key" + c] = [c.toLowerCase(), c];
})();
