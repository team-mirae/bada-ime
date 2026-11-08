import Cocoa
import IOKit.hid

/// Keyboard keys macOS doesn't turn into key events of their own (spec §6.6): a PC keyboard's 変換
/// (HID usage 0x8A) comes through as nothing, and カタカナ (0x88) as かな. bada ime reads them at the
/// HID level, which needs Input Monitoring.
///
/// A profile key recorded this way is stored as `HIDKeys.base | usage` (keyboard usage page 7).
/// When such a key goes down, the next かな / 英数 key event within `window` is taken as that key
/// (カタカナ arrives as かな); if none comes, the HID key acts on its own (変換 arrives as nothing).
enum HIDKeys {
    static let base = 0x7_0000
    private static let window = 0.08
    /// Usages read here: the keys PC keyboards add around the space bar and above.
    static let names: [Int: String] = [
        0x87: "ろ (HID 0x87)", 0x88: "カタカナ/ひらがな (HID 0x88)", 0x89: "¥ (HID 0x89)", 0x8A: "変換 (HID 0x8A)",
        0x8B: "無変換 (HID 0x8B)", 0x92: "LANG3 (HID 0x92)", 0x93: "LANG4 (HID 0x93)", 0x94: "LANG5 (HID 0x94)",
    ]

    private static var manager: IOHIDManager?
    /// Set while Settings records: the next special key goes here.
    static var recorder: ((Int) -> Void)?
    /// A special key just pressed, not yet claimed by a key event.
    private static var pending: (code: Int, at: Date, recorded: Bool)?

    static func isHID(_ code: Int) -> Bool { code >= base }

    static func name(_ code: Int) -> String? {
        isHID(code) ? names[code - base] ?? String(format: "HID 0x%02X", code - base) : nil
    }

    /// Starts listening, asking for Input Monitoring if `ask` (once; macOS remembers the answer).
    static func start(ask: Bool) {
        guard manager == nil else { return }
        if IOHIDCheckAccess(kIOHIDRequestTypeListenEvent) != kIOHIDAccessTypeGranted {
            guard ask else { return }
            IOHIDRequestAccess(kIOHIDRequestTypeListenEvent)
            DispatchQueue.main.asyncAfter(deadline: .now() + 3) { start(ask: false) }
            return
        }
        let m = IOHIDManagerCreate(kCFAllocatorDefault, IOOptionBits(kIOHIDOptionsTypeNone))
        IOHIDManagerSetDeviceMatching(m, [kIOHIDDeviceUsagePageKey: kHIDPage_GenericDesktop,
                                          kIOHIDDeviceUsageKey: kHIDUsage_GD_Keyboard] as CFDictionary)
        IOHIDManagerRegisterInputValueCallback(m, { _, _, _, value in HIDKeys.value(value) }, nil)
        IOHIDManagerScheduleWithRunLoop(m, CFRunLoopGetMain(), CFRunLoopMode.commonModes.rawValue)
        let result = IOHIDManagerOpen(m, IOOptionBits(kIOHIDOptionsTypeNone))
        // Exclusive access: some keyboard is seized (Karabiner-Elements seizes the physical ones and
        // types through its own virtual keyboard). The others, that virtual one included, are open.
        guard result == kIOReturnSuccess || result == kIOReturnExclusiveAccess else {
            DiagnosticLog.note("HID keys: couldn't open (\(result))")
            return
        }
        manager = m
        DiagnosticLog.note(result == kIOReturnSuccess ? "HID keys: on" : "HID keys: on (some keyboards are seized by another app)")
    }

    /// On when profiles are and a profile key is a HID key.
    static func check(_ settings: BadaSettings = BadaSettings.current) {
        if settings.use_profiles && settings.profile_keys.values.contains(where: isHID) {
            start(ask: false)
        }
    }

    private static func value(_ value: IOHIDValue) {
        let element = IOHIDValueGetElement(value)
        guard IOHIDElementGetUsagePage(element) == kHIDPage_KeyboardOrKeypad,
              IOHIDValueGetIntegerValue(value) == 1 else { return }
        let usage = Int(IOHIDElementGetUsage(element))
        guard names[usage] != nil else { return }
        DiagnosticLog.note(String(format: "HID keys: usage 0x%02X", usage))
        let code = base | usage
        if let recorder {
            Self.recorder = nil
            pending = (code, Date(), true)   // its かな / 英数 echo, if any, is only swallowed
            recorder(code)
            return
        }
        guard let name = LanguageKeyTap.profileKeys[code] else { return }
        pending = (code, Date(), false)
        // No key event claimed it in time: it arrives as nothing (変換), so act here.
        DispatchQueue.main.asyncAfter(deadline: .now() + window) {
            guard let p = pending, p.code == code else { return }
            pending = nil
            LanguageKeys.profile(name, shift: NSEvent.modifierFlags.contains(.shift))
        }
    }

    /// A かな or 英数 key event: the HID key that caused it, if one went down just before (カタカナ
    /// arrives as かな). Claims it, so it doesn't act twice. `recorded`: it was just recorded, so the
    /// event should do nothing.
    static func claim() -> (code: Int, recorded: Bool)? {
        guard let p = pending, Date().timeIntervalSince(p.at) < window else { return nil }
        pending = nil
        return (p.code, p.recorded)
    }
}
