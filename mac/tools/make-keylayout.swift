// Writes the keyboard layouts the input method shows (overrideKeyboard), so the Keyboard Viewer
// shows what bada ime types. Both are macOS's ABC layout as a JIS keyboard has it, with changes:
//   bada.keylayout (henkan): bada plain's (layout 16) legends on the unshifted and ⇧ layers: its
//     jamo and punctuation, and with ⇧ the syllables and digits. A key the engine passes on there
//     is typed as ABC types it, by the input method itself (Keyboards.passOn).
//   qwerty.keylayout (muhenkan): ろ is \ and ⇧ろ _, as the engine types it (ABC gives _ for both).
// Every other layer (⌥, ⌘, ⌃) and key is ABC's, so shortcuts are unchanged.
//
//   (Makefile) build/make-keylayout ../docs/layouts/16-bada-plain.json build/
import Carbon
import Foundation

let args = CommandLine.arguments
guard args.count == 3 else { FileHandle.standardError.write("usage: make-keylayout 16-bada-plain.json outdir/\n".data(using: .utf8)!); exit(2) }

// The layout's legends per key name: [unshifted, ⇧]. A token of role "a" is its consonant and ㅏ,
// one of role "n" its vowel on ㅇ (docs/layouts/16-bada-plain.md): the legend is the syllable.
let initials = Array("ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ"), vowels = Array("ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ")
func syllable(_ initial: Character, _ vowel: Character) -> String? {
    guard let i = initials.firstIndex(of: initial), let v = vowels.firstIndex(of: vowel) else { return nil }
    return UnicodeScalar(UInt32(0xAC00 + (i * 21 + v) * 28)).map { String($0) }
}
func legend(_ token: [String: String]?) -> String? {
    guard let token, let text = token["t"], let jamo = text.first else { return nil }
    switch token["r"] {
    case "a": return syllable(jamo, "ㅏ")
    case "n": return syllable("ㅇ", jamo)
    default: return text
    }
}
let json = try JSONSerialization.jsonObject(with: Data(contentsOf: URL(fileURLWithPath: args[1]))) as! [String: Any]
var bada: [String: [String?]] = [:]
for (name, layers) in json["keys"] as! [String: [String: [[String: String]]]] {
    bada[name] = [legend(layers["base"]?.first), legend(layers["shift"]?.first)]
}

// ABC, as a JIS keyboard (type 18) types it
let filter = [kTISPropertyInputSourceID as String: "com.apple.keylayout.ABC"] as CFDictionary
let abc = (TISCreateInputSourceList(filter, true).takeRetainedValue() as! [TISInputSource])[0]
let data = Unmanaged<CFData>.fromOpaque(TISGetInputSourceProperty(abc, kTISPropertyUnicodeKeyLayoutData)).takeUnretainedValue()
let layout = unsafeBitCast(CFDataGetBytePtr(data), to: UnsafePointer<UCKeyboardLayout>.self)
func abcChars(_ code: Int, _ mods: Int) -> String {
    var dead: UInt32 = 0, len = 0
    var buf = [UniChar](repeating: 0, count: 8)
    UCKeyTranslate(layout, UInt16(code), UInt16(kUCKeyActionDisplay), UInt32((mods >> 8) & 0xFF), 18,
                   OptionBits(kUCKeyTranslateNoDeadKeysBit), &dead, buf.count, &len, &buf)
    return String(utf16CodeUnits: buf, count: len)
}

// The layers: its modifiers, the Carbon modifiers to translate ABC with, and which of the layout's
// own layers (0 unshifted, 1 ⇧) it takes, if any. Caps Lock has a layer of its own (ABC's capitals;
// the layout's own unshifted keys, so ろ stays \ and the jamo stay jamo): without it, Caps Lock
// would type lowercase (spec §6.7).
let maps: [(String, Int, Int?)] = [
    ("<modifier keys=\"\"/>", 0, 0),
    ("<modifier keys=\"anyShift caps?\"/>", shiftKey, 1),
    ("<modifier keys=\"caps\"/>", alphaLock, 0),
    ("<modifier keys=\"caps? anyOption\"/>", optionKey, nil),
    ("<modifier keys=\"anyShift caps? anyOption\"/>", shiftKey | optionKey, nil),
    ("<modifier keys=\"anyShift? caps? anyOption? command\"/>", cmdKey, nil),
    ("<modifier keys=\"anyShift? caps? anyOption? command? anyControl\"/>", controlKey, nil),
]
func xml(_ s: String) -> String {
    s.unicodeScalars.map { u in
        u.value < 0x20 || u.value == 0x7F || "\"&'<>".unicodeScalars.contains(u) ? String(format: "&#x%04X;", u.value) : String(u)
    }.joined()
}

/// A layout: ABC with `change(code, layer)` where it returns something (layers 0 and 1 only: the
/// unshifted, ⇧ and Caps Lock layers).
func write(name: String, id: Int, _ change: (Int, Int) -> String?) throws {
    var out = """
    <?xml version="1.1" encoding="UTF-8"?>
    <!DOCTYPE keyboard SYSTEM "file://localhost/System/Library/DTDs/KeyboardLayout.dtd">
    <!-- Written by mac/tools/make-keylayout.swift, for the Keyboard Viewer. -->
    <keyboard group="126" id="\(id)" name="\(name)" maxout="2">
        <layouts>
            <layout first="0" last="255" mapSet="all" modifiers="mods"/>
        </layouts>
        <modifierMap id="mods" defaultIndex="0">

    """
    for (i, (mod, _, _)) in maps.enumerated() { out += "        <keyMapSelect mapIndex=\"\(i)\">\(mod)</keyMapSelect>\n" }
    out += "    </modifierMap>\n    <keyMapSet id=\"all\">\n"
    for (i, (_, mods, own)) in maps.enumerated() {
        out += "        <keyMap index=\"\(i)\">\n"
        for code in 0..<128 {
            var s = abcChars(code, mods)
            if let own, let changed = change(code, own) { s = changed }
            if !s.isEmpty { out += "            <key code=\"\(code)\" output=\"\(xml(s))\"/>\n" }
        }
        out += "        </keyMap>\n"
    }
    out += "    </keyMapSet>\n</keyboard>\n"
    try out.write(toFile: args[2] + "/\(name).keylayout", atomically: true, encoding: .utf8)
}

try write(name: "bada", id: -20513) { code, layer in keyNames[code].flatMap { bada[$0]?[layer] ?? nil } }
try write(name: "qwerty", id: -20514) { code, layer in code == kVK_JIS_Underscore ? (layer == 0 ? "\\" : "_") : nil }
