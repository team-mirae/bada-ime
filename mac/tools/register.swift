// Registers bada ime with Text Input Sources so it shows in System Settings without logging out.
// Then both modes are enabled, always: bada ime has exactly two (spec §4.1). Modes already on are
// left alone (turning them off and on again, as JIS Moonshot does, left only one of the two on).
// Usage: swift register.swift <BadaIME.app>
import Carbon
import Foundation

let path = CommandLine.arguments[1]
let bundleID = Bundle(path: path)?.bundleIdentifier ?? "me.sjun.inputmethod.Bada"
let modes = [bundleID + ".Muhenkan", bundleID + ".Henkan"]

func sources(enabledOnly: Bool) -> [TISInputSource] {
    let query = [kTISPropertyBundleID as String: bundleID] as CFDictionary
    return TISCreateInputSourceList(query, !enabledOnly)?.takeRetainedValue() as? [TISInputSource] ?? []
}
func id(_ s: TISInputSource) -> String {
    TISGetInputSourceProperty(s, kTISPropertyInputSourceID).map { Unmanaged<CFString>.fromOpaque($0).takeUnretainedValue() as String } ?? ""
}
func settle(_ seconds: Double) { CFRunLoopRunInMode(.defaultMode, seconds, false) }

let before = Set(sources(enabledOnly: true).map(id)).intersection(modes)
guard TISRegisterInputSource(URL(fileURLWithPath: path) as CFURL) == noErr else { print("TISRegisterInputSource failed"); exit(1) }
settle(1)
let all = sources(enabledOnly: false).filter { modes.contains(id($0)) }
// No off-and-on cycle for modes already enabled: with two modes, macOS then kept only one.
// One mode at a time, each given time to land: enabled back to back, macOS kept only one of them.
for _ in 0..<5 {
    let on = Set(sources(enabledOnly: true).map(id))
    let missing = all.filter { !on.contains(id($0)) }
    if missing.isEmpty { break }
    for source in missing {
        TISEnableInputSource(source)
        settle(1.5)
    }
}
let now = Set(sources(enabledOnly: true).map(id)).intersection(modes)
print(now.isEmpty
    ? "Registered. Add bada (muhenkan) under English and bada (henkan) under Japanese in System Settings → Keyboard → Input Sources (log out and in if they aren't listed)."
    : "Registered; enabled: \(now.sorted().joined(separator: ", ")).")
