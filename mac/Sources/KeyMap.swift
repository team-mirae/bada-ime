import Carbon.HIToolbox

/// macOS key codes → canonical key names (`KeyboardEvent.code`), as a JIS keyboard sends them.
let keyNames: [Int: String] = [
    kVK_ANSI_Grave: "Backquote",
    kVK_ANSI_1: "Digit1", kVK_ANSI_2: "Digit2", kVK_ANSI_3: "Digit3", kVK_ANSI_4: "Digit4", kVK_ANSI_5: "Digit5",
    kVK_ANSI_6: "Digit6", kVK_ANSI_7: "Digit7", kVK_ANSI_8: "Digit8", kVK_ANSI_9: "Digit9", kVK_ANSI_0: "Digit0",
    kVK_ANSI_Minus: "Minus", kVK_ANSI_Equal: "Equal", kVK_JIS_Yen: "IntlYen",
    kVK_ANSI_Q: "KeyQ", kVK_ANSI_W: "KeyW", kVK_ANSI_E: "KeyE", kVK_ANSI_R: "KeyR", kVK_ANSI_T: "KeyT",
    kVK_ANSI_Y: "KeyY", kVK_ANSI_U: "KeyU", kVK_ANSI_I: "KeyI", kVK_ANSI_O: "KeyO", kVK_ANSI_P: "KeyP",
    kVK_ANSI_LeftBracket: "BracketLeft", kVK_ANSI_RightBracket: "BracketRight",
    kVK_ANSI_A: "KeyA", kVK_ANSI_S: "KeyS", kVK_ANSI_D: "KeyD", kVK_ANSI_F: "KeyF", kVK_ANSI_G: "KeyG",
    kVK_ANSI_H: "KeyH", kVK_ANSI_J: "KeyJ", kVK_ANSI_K: "KeyK", kVK_ANSI_L: "KeyL",
    kVK_ANSI_Semicolon: "Semicolon", kVK_ANSI_Quote: "Quote", kVK_ANSI_Backslash: "Backslash",
    kVK_ANSI_Z: "KeyZ", kVK_ANSI_X: "KeyX", kVK_ANSI_C: "KeyC", kVK_ANSI_V: "KeyV", kVK_ANSI_B: "KeyB",
    kVK_ANSI_N: "KeyN", kVK_ANSI_M: "KeyM",
    kVK_ANSI_Comma: "Comma", kVK_ANSI_Period: "Period", kVK_ANSI_Slash: "Slash", kVK_JIS_Underscore: "IntlRo",
    kVK_F6: "F6", kVK_F7: "F7", kVK_F8: "F8", kVK_F9: "F9", kVK_F10: "F10",
    kVK_Space: "Space", kVK_Return: "Enter", kVK_ANSI_KeypadEnter: "Enter", kVK_Delete: "Backspace",
    kVK_Tab: "Tab", kVK_Escape: "Escape",
    kVK_UpArrow: "ArrowUp", kVK_DownArrow: "ArrowDown", kVK_LeftArrow: "ArrowLeft", kVK_RightArrow: "ArrowRight",
    kVK_JIS_Kana: "Lang1", kVK_JIS_Eisu: "Lang2",
]

func keyName(_ keyCode: UInt16) -> String {
    keyNames[Int(keyCode)] ?? "Other"
}
