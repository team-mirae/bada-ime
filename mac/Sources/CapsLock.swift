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
    /// The press being handled is macOS's own (another input source): left alone.
    private static var native = false

    /// Caps Lock is on, for what bada ime types (spec §6.2): as bada ime holds it when it handles
    /// Caps Lock (macOS's key events don't carry that), else as the key event says.
    static func isOn(_ flags: NSEvent.ModifierFlags) -> Bool {
        isHandled ? on : flags.contains(.capsLock)
    }
    /// Until when a Caps Lock change that disagrees with `on` is macOS's own toggle, to undo.
    private static var enforceUntil = Date.distantPast
    private static var keyDown = false

    /// macOS's Caps Lock state, as IOHIDSystem has it (CGEventSource's flags can lag behind it, as
    /// with Karabiner-Elements' virtual keyboard).
    static var systemState: Bool {
        var state = false
        withSystem { IOHIDGetModifierLockState($0, Int32(kIOHIDCapsLockState), &state) }
        return state
    }

    private static func withSystem(_ body: (io_connect_t) -> Void) {
        let service = IOServiceGetMatchingService(kIOMainPortDefault, IOServiceMatching(kIOHIDSystemClass))
        guard service != 0 else { return }
        defer { IOObjectRelease(service) }
        var connect: io_connect_t = 0
        guard IOServiceOpen(service, mach_task_self_, UInt32(kIOHIDParamConnectType), &connect) == KERN_SUCCESS else { return }
        defer { IOServiceClose(connect) }
        body(connect)
    }

    /// Sets macOS's Caps Lock state (and so its LED).
    private static func setSystem(_ value: Bool) {
        DiagnosticLog.note("Caps Lock: set system \(value)")
        withSystem { IOHIDSetModifierLockState($0, Int32(kIOHIDCapsLockState), value) }
    }

    /// Holds Caps Lock at `value`: set, and the LEDs set directly too, even when macOS already reads
    /// it so (a keyboard's LED can lag behind, as through Karabiner-Elements).
    private static func hold(_ value: Bool) {
        on = value
        setSystem(value)
        HIDKeys.setCapsLED(value)
        BadaState.refresh()
    }

    /// The physical key, from HIDKeys: down or up. `on` is still what it was before the press: a
    /// change macOS reports first waits for this (`changed`).
    static func key(down: Bool) {
        guard isHandled else { return }
        DiagnosticLog.note("Caps Lock: key \(down ? "down" : "up"), held \(on), system \(systemState)")
        if native {
            if !down { native = false }
            return
        }
        if !down {
            keyDown = false
            enforceUntil = Date().addingTimeInterval(0.5)
            if systemState != on { setSystem(on) }
            for delay in [0.3, 1.0] {
                DispatchQueue.main.asyncAfter(deadline: .now() + delay) {
                    DiagnosticLog.note("Caps Lock: +\(delay)s held \(on), system \(systemState), LEDs \(HIDKeys.capsLEDs())")
                }
            }
            return
        }
        keyDown = true
        enforceUntil = .distantFuture
        // Another input source (ABC in a password field): Caps Lock is macOS's own toggle, untouched.
        guard let mode = Modes.selectedNow() else {
            keyDown = false
            enforceUntil = .distantPast
            native = true
            on.toggle()
            BadaState.refresh()
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
        if value == on { DiagnosticLog.note("Caps Lock: macOS says \(value), as held"); return true }
        DiagnosticLog.note("Caps Lock: macOS says \(value), held \(on)\(keyDown || Date() < enforceUntil ? " (the key: undone)" : "")")
        if keyDown || Date() < enforceUntil {
            DispatchQueue.main.async { if systemState != on { setSystem(on) } }
            return false
        }
        // macOS often reports its own toggle before the key's HID report arrives: wait for it before
        // taking the change as another app's. Swallowed meanwhile.
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            if keyDown || Date() < enforceUntil {
                if systemState != on { setSystem(on) }
                return
            }
            // Not the key: another app. In henkan, off unless Caps Lock is respected.
            if setting != .respect && Modes.selectedNow() == .henkan {
                DiagnosticLog.note("Caps Lock: another app turned it on in henkan: off")
                hold(false)
            } else {
                on = value
                BadaState.refresh()
            }
        }
        return false
    }

    /// The mode changed: leaving muhenkan forgets Caps Lock, unless it's respected.
    static func modeChanged(to mode: BadaMode) {
        guard isHandled, setting != .respect, mode != .muhenkan else { return }
        hold(false)
    }

    /// The Caps Lock LED lit for a moment, set directly on each keyboard (a mode switch by Caps Lock
    /// always leaves Caps Lock off).
    static func blink() {
        HIDKeys.setCapsLED(true)
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
