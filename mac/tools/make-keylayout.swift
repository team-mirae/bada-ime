// Writes the keyboard layouts the input method shows (overrideKeyboard), so the Keyboard Viewer
// shows what bada ime types. Both are macOS's ABC layout as a JIS keyboard has it, with changes:
//   bada.keylayout (henkan): bada's (layout 13) jamo on the unshifted and ⇧ layers of the keys
//     that carry them. A key the engine passes on there is typed as ABC types it, by the input
//     method itself (Keyboards.passOn).
//   qwerty.keylayout (muhenkan): ろ is \ and ⇧ろ _, as the engine types it (ABC gives _ for both).
// Every other layer (⌥, ⌘, ⌃) and key is ABC's, so shortcuts are unchanged.
//
//   (Makefile) build/make-keylayout ../docs/layouts/13-bada.json build/
import Carbon
import Foundation

let args = CommandLine.arguments
guard args.count == 3 else { FileHandle.standardError.write("usage: make-keylayout 13-bada.json outdir/\n".data(using: .utf8)!); exit(2) }

// bada's legends per key name: [unshifted, ⇧]
let json = try JSONSerialization.jsonObject(with: Data(contentsOf: URL(fileURLWithPath: args[1]))) as! [String: Any]
var bada: [String: [String?]] = [:]
for (name, layers) in json["keys"] as! [String: [String: [[String: String]]]] {
    bada[name] = [layers["base"]?.first?["t"], layers["shift"]?.first?["t"]]
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

// The layers: index, its modifiers, the Carbon modifiers to translate ABC with
let maps: [(String, Int)] = [
    ("<modifier keys=\"caps?\"/>", 0),
    ("<modifier keys=\"anyShift caps?\"/>", shiftKey),
    ("<modifier keys=\"caps? anyOption\"/>", optionKey),
    ("<modifier keys=\"anyShift caps? anyOption\"/>", shiftKey | optionKey),
    ("<modifier keys=\"anyShift? caps? anyOption? command\"/>", cmdKey),
    ("<modifier keys=\"anyShift? caps? anyOption? command? anyControl\"/>", controlKey),
]
func xml(_ s: String) -> String {
    s.unicodeScalars.map { u in
        u.value < 0x20 || u.value == 0x7F || "\"&'<>".unicodeScalars.contains(u) ? String(format: "&#x%04X;", u.value) : String(u)
    }.joined()
}

/// A layout: ABC with `change(code, layer)` where it returns something (layers 0 and 1 only).
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
    for (i, (mod, _)) in maps.enumerated() { out += "        <keyMapSelect mapIndex=\"\(i)\">\(mod)</keyMapSelect>\n" }
    out += "    </modifierMap>\n    <keyMapSet id=\"all\">\n"
    for (i, (_, mods)) in maps.enumerated() {
        out += "        <keyMap index=\"\(i)\">\n"
        for code in 0..<128 {
            var s = abcChars(code, mods)
            if i < 2, let changed = change(code, i) { s = changed }
            if !s.isEmpty { out += "            <key code=\"\(code)\" output=\"\(xml(s))\"/>\n" }
        }
        out += "        </keyMap>\n"
    }
    out += "    </keyMapSet>\n</keyboard>\n"
    try out.write(toFile: args[2] + "/\(name).keylayout", atomically: true, encoding: .utf8)
}

try write(name: "bada", id: -20513) { code, layer in keyNames[code].flatMap { bada[$0]?[layer] ?? nil } }
try write(name: "qwerty", id: -20514) { code, layer in code == kVK_JIS_Underscore ? (layer == 0 ? "\\" : "_") : nil }
