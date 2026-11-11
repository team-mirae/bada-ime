import Carbon.HIToolbox
import InputMethodKit

/// The keyboard layout macOS uses while bada ime is selected, which is also what the Keyboard Viewer
/// shows (tools/make-keylayout.swift). Muhenkan: qwerty.keylayout, ABC with ろ as \\ and ⇧ろ _ (keys the
/// engine passes on type JIS characters through it). Henkan: bada.keylayout, ABC with bada's
/// (layout 13) jamo on the unshifted and ⇧ layers, whatever layouts are in use.
enum Keyboards {
    static let abc = "com.apple.keylayout.ABC"
    static let bada = "me.sjun.inputmethod.Bada.keylayout.bada"
    static let qwerty = "me.sjun.inputmethod.Bada.keylayout.qwerty"

    /// Sets the mode's keyboard, now and once macOS has finished selecting the mode (which may put
    /// the mode's own keyboard back). Our layouts are asked for by name, not looked up first: the
    /// input method's process may not see a layout macOS has just registered.
    static func show(for mode: BadaMode, in client: IMKTextInput) {
        let name = mode == .henkan ? bada : qwerty
        client.overrideKeyboard(withKeyboardNamed: name)
        for delay in [0.15, 0.6] {
            DispatchQueue.main.asyncAfter(deadline: .now() + delay) {
                guard Modes.current == mode else { return }
                client.overrideKeyboard(withKeyboardNamed: name)
            }
        }
    }

    /// What ABC types for a key that bada.keylayout gives a jamo on this layer, or nil: unshifted,
    /// every key named in KeyMap that bada (13) takes; with ⇧, J K L ; ¥ (docs/bada-layout.md §2).
    static func passOn(code: Int, shift: Bool) -> String? {
        guard let name = keyNames[code], badaKeys.contains(name) else { return nil }
        if shift && !badaShifted.contains(code) { return nil }
        return translate(abc, code, shift ? UInt32(shiftKey >> 8) : 0)
    }

    /// A letter key's capital, as ABC types it with Caps Lock; nil for other keys. In muhenkan the
    /// input method types it itself while Caps Lock is on (spec §6.7): the Caps Lock bada ime sets
    /// lights the LED but isn't on macOS's key events.
    static func capital(code: Int) -> String? {
        guard let name = keyNames[code], name.count == 4, name.hasPrefix("Key") else { return nil }
        return translate(abc, code, UInt32(alphaLock >> 8))
    }

    private static let badaKeys: Set<String> = Set(
        ["Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8", "Digit9", "Digit0", "Minus", "Equal",
         "IntlYen", "BracketLeft", "BracketRight", "Semicolon", "Quote", "Backslash", "Comma", "Period", "Slash", "IntlRo"]
        + "QWERTYUIOPASDFGHJKLZXCVBNM".map { "Key\($0)" })
    private static let badaShifted: Set<Int> = [kVK_ANSI_J, kVK_ANSI_K, kVK_ANSI_L, kVK_ANSI_Semicolon, kVK_JIS_Yen]

    private static func source(_ id: String) -> TISInputSource? {
        let filter = [kTISPropertyInputSourceID as String: id] as CFDictionary
        return (TISCreateInputSourceList(filter, true)?.takeRetainedValue() as? [TISInputSource])?.first
    }

    private static func translate(_ id: String, _ code: Int, _ mods: UInt32) -> String? {
        guard let src = source(id), let ptr = TISGetInputSourceProperty(src, kTISPropertyUnicodeKeyLayoutData) else { return nil }
        let data = Unmanaged<CFData>.fromOpaque(ptr).takeUnretainedValue()
        guard let bytes = CFDataGetBytePtr(data) else { return nil }
        let layout = UnsafeRawPointer(bytes).assumingMemoryBound(to: UCKeyboardLayout.self)
        var dead: UInt32 = 0, len = 0
        var buf = [UniChar](repeating: 0, count: 8)
        let status = UCKeyTranslate(layout, UInt16(code), UInt16(kUCKeyActionDown), mods, UInt32(LMGetKbdType()),
                                    OptionBits(kUCKeyTranslateNoDeadKeysBit), &dead, buf.count, &len, &buf)
        return status == noErr && len > 0 ? String(utf16CodeUnits: buf, count: len) : nil
    }
}
