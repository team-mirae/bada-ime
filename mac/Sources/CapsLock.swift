import Cocoa
import IOKit.hid
import IOKit.hidsystem

/// Caps Lock, handled by bada ime (spec §6.7), without remapping the key: every press is read at the
/// HID level (`HIDKeys`, Input Monitoring), bada ime decides what Caps Lock should be and sets it,
/// and while the key is down (and a moment after) the toggle macOS makes on its own is undone. The
/// key tap (Accessibility) swallows the Caps Lock events that disagree with bada ime, so apps never
/// see macOS's own toggle. Needs both; without them Caps Lock is macOS's.
///
/// - toggle (default): Caps Lock switches mode, as MEH+かな, and blinks the LED; ⇧ ⌃ ⌥ or ⌘ +
///   Caps Lock is Caps Lock, in muhenkan only, and forgotten when muhenkan is left.
/// - transient: Caps Lock is Caps Lock in muhenkan, forgotten when muhenkan is left; in henkan it's
///   off, and turned back off if another app turns it on.
/// - respect: Caps Lock is Caps Lock everywhere, and lasts until logout (macOS keeps it until then).
enum CapsLock {
    enum Setting: String, CaseIterable {
        case toggle, transient, respect

        var title: String {
            switch self {
            case .toggle: return "Switch mode (⇧ ⌃ ⌥ ⌘ + Caps Lock: Caps Lock, in muhenkan)"
            case .transient: return "Caps Lock in muhenkan only, forgotten when you leave it"
            case .respect: return "Caps Lock everywhere, until you log out"
            }
        }
    }

    /// What Caps Lock does to muhenkan typing (spec §6.2).
    enum Rule: String, CaseIterable {
        case allcaps, uppersnake

        var title: String {
            switch self {
            case .allcaps: return "ALL CAPS: letters in capitals"
            case .uppersnake: return "UPPER_SNAKE: capitals, Space types _ (⌥Space a space)"
            }
        }
    }

    static var setting: Setting { Setting(rawValue: BadaSettings.current.caps_lock) ?? .toggle }
    static var rule: Rule { Rule(rawValue: BadaSettings.current.caps_rule) ?? .allcaps }

    /// Handled: the key tap is on and Input Monitoring is granted (HIDKeys is listening).
    static var isHandled: Bool { LanguageKeyTap.isOn && HIDKeys.isListening }

    /// Caps Lock as bada ime holds it.
    private static var on = systemState
    /// Until when a Caps Lock change that disagrees with `on` is macOS's own toggle, to undo.
    private static var enforceUntil = Date.distantPast
    private static var keyDown = false

    static var systemState: Bool { CGEventSource.flagsState(.hidSystemState).contains(.maskAlphaShift) }

    /// Sets macOS's Caps Lock state (and so its LED).
    private static func setSystem(_ value: Bool) {
        let service = IOServiceGetMatchingService(kIOMainPortDefault, IOServiceMatching(kIOHIDSystemClass))
        guard service != 0 else { return }
        defer { IOObjectRelease(service) }
        var connect: io_connect_t = 0
        guard IOServiceOpen(service, mach_task_self_, UInt32(kIOHIDParamConnectType), &connect) == KERN_SUCCESS else { return }
        defer { IOServiceClose(connect) }
        IOHIDSetModifierLockState(connect, Int32(kIOHIDCapsLockState), value)
    }

    private static func hold(_ value: Bool) {
        on = value
        if systemState != value { setSystem(value) }
    }

    /// The physical key, from HIDKeys: down or up.
    static func key(down: Bool) {
        guard isHandled else { return }
        if !down {
            keyDown = false
            enforceUntil = Date().addingTimeInterval(0.5)
            if systemState != on { setSystem(on) }
            return
        }
        keyDown = true
        enforceUntil = .distantFuture
        // Another input source (ABC in a password field): Caps Lock as macOS has it.
        guard let mode = Modes.selectedNow() else {
            hold(!on)
            return
        }
        let flags = CGEventSource.flagsState(.combinedSessionState)
        let modified = !flags.intersection([.maskShift, .maskControl, .maskAlternate, .maskCommand]).isEmpty
        switch setting {
        case .toggle where !modified:
            hold(false)   // muhenkan-transient: off in henkan, and forgotten leaving muhenkan
            LanguageKeys.toggle()
            blink()
        case .toggle, .transient:
            hold(mode == .muhenkan ? !on : false)
        case .respect:
            hold(!on)
        }
    }

    /// From the key tap: a Caps Lock change macOS reports. Returns false to swallow it (macOS's own
    /// toggle, undone).
    static func changed(to value: Bool) -> Bool {
        guard isHandled else { return true }
        if value == on { return true }
        if keyDown || Date() < enforceUntil {
            DispatchQueue.main.async { if systemState != on { setSystem(on) } }
            return false
        }
        // Not the key: another app. In henkan, off unless Caps Lock is respected.
        if setting != .respect && Modes.selectedNow() == .henkan {
            DispatchQueue.main.async { hold(false) }
            return false
        }
        on = value
        return true
    }

    /// The mode changed: leaving muhenkan forgets Caps Lock, unless it's respected.
    static func modeChanged(to mode: BadaMode) {
        guard isHandled, setting != .respect, mode != .muhenkan else { return }
        if on || systemState { hold(false) }
    }

    /// The Caps Lock LED, lit or dark against the state for a moment, set directly on each keyboard.
    static func blink() {
        HIDKeys.setCapsLED(!on)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) { HIDKeys.setCapsLED(on) }
    }

    /// When bada ime starts handling it: in sync with macOS, and off in henkan unless respected.
    static func start() {
        on = systemState
        if setting != .respect, Modes.selectedNow() == .henkan, on { hold(false) }
        if UserDefaults(suiteName: "com.apple.HIToolbox")?.integer(forKey: "TISRomanSwitchState") == 1 {
            DiagnosticLog.note("Caps Lock: macOS's \"Use Caps Lock to switch to and from ABC\" is on; turn it off for bada ime's Caps Lock")
        }
    }
}
