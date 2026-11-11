import Carbon
import Cocoa
import InputMethodKit

/// bada ime's two input modes (spec §4.1): Bada Muhenkan (English, Roman script) and Bada Henkan
/// (Japanese). Layouts inside a mode are the engine's business, not input sources.
enum BadaMode: String {
    case muhenkan = "Muhenkan"
    case henkan = "Henkan"

    var sourceID: String { Modes.bundleID + "." + rawValue }
    var other: BadaMode { self == .muhenkan ? .henkan : .muhenkan }
}

enum Modes {
    static let bundleID = Bundle.main.bundleIdentifier ?? "me.sjun.inputmethod.Bada"

    /// The mode bada ime is in, as last selected or reported.
    static var current: BadaMode = selectedNow() ?? .henkan {
        didSet { if current != oldValue { CapsLock.modeChanged(to: current) } }
    }
    /// The henkan profile in use (spec §6.6): which language key brought henkan mode up.
    static var profile = BadaProfiles.macKana

    static func mode(sourceID: String) -> BadaMode? {
        [BadaMode.muhenkan, .henkan].first { $0.sourceID == sourceID }
    }

    /// Which of our modes macOS has selected now; nil: another input source.
    static func selectedNow() -> BadaMode? {
        let source = TISCopyCurrentKeyboardInputSource().takeRetainedValue()
        guard let value = TISGetInputSourceProperty(source, kTISPropertyInputSourceID) else { return nil }
        return mode(sourceID: Unmanaged<CFString>.fromOpaque(value).takeUnretainedValue() as String)
    }

    static func isEnabled(_ mode: BadaMode) -> Bool {
        let query = [kTISPropertyInputSourceID as String: mode.sourceID] as CFDictionary
        return (TISCreateInputSourceList(query, false)?.takeRetainedValue() as? [TISInputSource])?.isEmpty == false
    }

    /// Selects a mode's input source. With one of ours selected and a session, the session switches
    /// (IMK's own way; TIS from the input method's process can cut the app's session off).
    static func select(_ mode: BadaMode, in client: IMKTextInput? = nil) {
        current = mode
        if let client, selectedNow() != nil {
            client.selectMode(mode.sourceID)
            return
        }
        let query = [kTISPropertyInputSourceID as String: mode.sourceID] as CFDictionary
        if let source = (TISCreateInputSourceList(query, false)?.takeRetainedValue() as? [TISInputSource])?.first {
            TISSelectInputSource(source)
        } else {
            DiagnosticLog.note("selecting \(mode.rawValue) failed: not added in System Settings")
        }
    }
}
