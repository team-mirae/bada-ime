import Cocoa
import InputMethodKit

/// かな and 英数 caught system-wide (spec §6.1), before any app sees them: macOS doesn't always pass
/// かな to an input method that is already in a Japanese mode, and some apps keep these keys.
/// Needs Accessibility; bada ime asks for it only from the input menu ("Allow 英数 and かな
/// Everywhere…"). It swallows the keys only while one of bada ime's modes is added.
///
/// - 英数: muhenkan mode.
/// - かな: henkan mode; already in henkan, the next menu entry (⇧かな: the previous).
/// - HYPER+かな (⌘⌃⌥⇧) and MEH+かな (⌃⌥⇧): the other mode (this build has one muhenkan layout and
///   one henkan, so going back to the last used one and cycling both land there).
/// - With henkan profiles on (spec §6.6): the keys recorded for 変換 and カタカナ select their profile.
/// - Caps Lock (spec §6.7): the changes macOS makes on its own are swallowed (CapsLock).
///
/// The tap never selects an input source itself. A key that changes mode is handed to macOS as a
/// plain 英数 or かな (`post`), so macOS's own switch does it; the input method acts on the key when
/// its mode is selected (`LanguageKeys.selected`, from activateServer / setValue).
enum LanguageKeyTap {
    private static var tap: CFMachPort?
    private static var polling = false
    /// Key code → profile, for the recorded 変換 / カタカナ keys (only with profiles on).
    private(set) static var profileKeys: [Int: String] = [:]
    /// Set while Settings records a key: the next key pressed anywhere goes here, and nowhere else.
    static var recorder: ((Int) -> Void)?
    static var isOn: Bool { tap != nil }
    /// Marks the 英数 / かな events bada ime posts, so the tap lets them through.
    private static let marker: Int64 = 0x6261_6461   // "bada"
    /// When the last one was posted: it may reach the input method as well as macOS's switch.
    private(set) static var lastPosted = Date.distantPast

    /// A plain 英数 or かな (no modifiers), for macOS to switch input source with.
    static func post(_ code: Int) {
        let source = CGEventSource(stateID: .privateState)
        source?.userData = marker
        for down in [true, false] {
            guard let event = CGEvent(keyboardEventSource: source, virtualKey: CGKeyCode(code), keyDown: down) else { continue }
            event.flags = []
            event.setIntegerValueField(.eventSourceUserData, value: marker)
            event.post(tap: .cgSessionEventTap)
        }
        lastPosted = Date()
    }

    static func refreshProfileKeys(_ settings: BadaSettings = BadaSettings.current) {
        profileKeys = [:]
        guard settings.use_profiles else { return }
        for (name, code) in settings.profile_keys where code != kVK_JIS_Kana && code != kVK_JIS_Eisu {
            profileKeys[code] = name
        }
    }

    static func check() {
        refreshProfileKeys()
        HIDKeys.check()
        if Modes.isEnabled(.henkan) || Modes.isEnabled(.muhenkan) {
            start(ask: false)
        }
    }

    static func start(ask: Bool) {
        guard tap == nil else { return }
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: ask] as CFDictionary
        guard AXIsProcessTrustedWithOptions(options) else {
            if ask && !polling {
                polling = true
                poll()
            }
            return
        }
        let mask = (1 << CGEventType.keyDown.rawValue) | (1 << CGEventType.keyUp.rawValue) | (1 << CGEventType.flagsChanged.rawValue)
        guard let port = CGEvent.tapCreate(tap: .cgSessionEventTap, place: .headInsertEventTap, options: .defaultTap,
                                           eventsOfInterest: CGEventMask(mask), callback: callback, userInfo: nil)
        else {
            DiagnosticLog.note("language key tap: couldn't create the tap")
            return
        }
        tap = port
        CFRunLoopAddSource(CFRunLoopGetMain(), CFMachPortCreateRunLoopSource(nil, port, 0), .commonModes)
        CGEvent.tapEnable(tap: port, enable: true)
        DiagnosticLog.note("language key tap: on")
        HIDKeys.check()   // Caps Lock needs the HID keys too
    }

    private static func poll() {
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            guard tap == nil else { polling = false; return }
            if AXIsProcessTrusted() { polling = false; start(ask: false) } else { poll() }
        }
    }

    private static let callback: CGEventTapCallBack = { _, type, event, _ in
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
            if let tap { CGEvent.tapEnable(tap: tap, enable: true) }
            return Unmanaged.passUnretained(event)
        }
        if event.getIntegerValueField(.eventSourceUserData) == marker { return Unmanaged.passUnretained(event) }
        let code = Int(event.getIntegerValueField(.keyboardEventKeycode))
        if type == .flagsChanged {
            guard code == kVK_CapsLock else { return Unmanaged.passUnretained(event) }
            return CapsLock.changed(to: event.flags.contains(.maskAlphaShift)) ? Unmanaged.passUnretained(event) : nil
        }
        if let recorder {
            if type == .keyDown {
                Self.recorder = nil
                DispatchQueue.main.async { recorder(code) }
            }
            return nil
        }
        let profileKey = profileKeys[code]
        guard code == kVK_JIS_Kana || code == kVK_JIS_Eisu || profileKey != nil else { return Unmanaged.passUnretained(event) }
        guard Modes.isEnabled(.henkan) || Modes.isEnabled(.muhenkan) else { return Unmanaged.passUnretained(event) }
        let flags = event.flags
        let press = type == .keyDown && event.getIntegerValueField(.keyboardEventAutorepeat) == 0
        if press {
            let shift = flags.contains(.maskShift), ctrl = flags.contains(.maskControl)
            let alt = flags.contains(.maskAlternate), cmd = flags.contains(.maskCommand)
            // A moment's wait: カタカナ arrives as かな, and the HID key that says so may come just after
            // (HIDKeys, whose own window is longer).
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.03) {
                var profileKey = profileKey
                if code == kVK_JIS_Kana || code == kVK_JIS_Eisu, let hid = HIDKeys.claim() {
                    if hid.recorded { return }
                    if let name = profileKeys[hid.code] { profileKey = name }
                }
                if let profileKey {
                    if !ctrl && !alt && !cmd { LanguageKeys.profile(profileKey, shift: shift) }
                } else if code == kVK_JIS_Eisu {
                    LanguageKeys.eisu()
                } else if ctrl && alt && shift {
                    LanguageKeys.toggle()  // HYPER (with ⌘) or MEH (without)
                } else if !ctrl && !alt && !cmd {
                    LanguageKeys.kana(shift: shift)
                }
            }
        }
        return nil
    }
}

/// What 英数, かな, HYPER+かな and MEH+かな do, from the tap or from a key that reached the input method.
enum LanguageKeys {
    /// A language key whose mode change was handed to macOS, waiting for that mode to be selected.
    private struct Press {
        let mode: BadaMode
        let profile: String?
        let at: Date
    }
    private static var pending: Press?
    /// How long macOS gets to switch before bada ime selects the mode itself.
    private static let fallback = 0.3

    static func eisu() {
        go(to: .muhenkan)
    }

    static func kana(shift: Bool) {
        profile(BadaProfiles.macKana, shift: shift)
    }

    /// かな, or a profile key (spec §6.6): henkan mode in that profile; already there, the next
    /// menu entry (⇧: the previous).
    static func profile(_ name: String, shift: Bool) {
        let name = BadaSettings.current.use_profiles ? name : BadaProfiles.macKana
        if Modes.selectedNow() == .henkan, let controller = BadaInputController.typing {
            controller.languageKey(name, shift: shift)
        } else {
            go(to: .henkan, profile: name)
        }
    }

    static func toggle() {
        go(to: (Modes.selectedNow() ?? Modes.current).other)
    }

    /// To `mode` (`profile`: in henkan, the profile to use). A change of mode is macOS's own switch,
    /// triggered by a plain 英数 / かな; if macOS hasn't switched within `fallback`, bada ime does.
    static func go(to mode: BadaMode, profile: String? = nil) {
        guard Modes.isEnabled(mode) else { return }
        guard LanguageKeyTap.isOn, Modes.selectedNow() != mode else {
            direct(mode, profile: profile)
            return
        }
        let press = Press(mode: mode, profile: profile, at: Date())
        pending = press
        LanguageKeyTap.post(mode == .muhenkan ? kVK_JIS_Eisu : kVK_JIS_Kana)
        DispatchQueue.main.asyncAfter(deadline: .now() + fallback) {
            guard let p = pending, p.at == press.at else { return }
            pending = nil
            if Modes.selectedNow() == mode {
                // Switched, but no session told us (no focused text field): note the profile.
                DiagnosticLog.note("switched to \(mode.rawValue) without a session")
                Modes.current = mode
                if let profile { Modes.profile = profile; BadaInputController.typing?.switchProfile(to: profile) }
            } else {
                DiagnosticLog.note("macOS didn't switch to \(mode.rawValue); selecting it directly")
                direct(mode, profile: profile)
            }
        }
    }

    /// Called when macOS has selected `mode` (activateServer / setValue): the henkan profile the
    /// language key that asked for it wants, if a key did.
    static func selected(_ mode: BadaMode) -> String? {
        guard let p = pending, p.mode == mode, Date().timeIntervalSince(p.at) < 1 else { return nil }
        pending = nil
        return p.profile
    }

    /// bada ime selecting the mode itself: without the tap, already in the mode, or as the fallback.
    private static func direct(_ mode: BadaMode, profile: String?) {
        if let profile { Modes.profile = profile }
        if Modes.selectedNow() != nil, let controller = BadaInputController.typing {
            controller.switchMode(to: mode)
        } else {
            Modes.select(mode)
        }
    }
}
