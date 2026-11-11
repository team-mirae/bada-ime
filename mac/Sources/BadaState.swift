import Carbon
import Cocoa

/// bada ime's state for other apps (spec §9; bada-compatible v1.1 §12.1), read-only: the engine's
/// `bada_state` as is, with the shell's own fields under "host", published whenever it changes.
///
/// - File: `~/Library/Application Support/me.sjun.inputmethod.Bada/state.json`, rewritten
///   atomically (a temporary file, then a rename).
/// - Distributed notification `me.sjun.inputmethod.Bada.state`, whose object is the same JSON.
///
/// Hammerspoon: `hs.pathwatcher` on the file or `hs.distributednotifications.new(fn,
/// "me.sjun.inputmethod.Bada.state")`. Karabiner-Elements: a script that reads the file and calls
/// `karabiner_cli --set-variables`. Nothing read from outside flows back into the engine.
///
/// In a password field (secure input) nothing typed is published: `staged`, `preedit` and `menu`
/// are empty, and `host.secure` is true.
enum BadaState {
    static let notification = Notification.Name((Bundle.main.bundleIdentifier ?? "me.sjun.inputmethod.Bada") + ".state")

    static let url: URL = {
        let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent(Bundle.main.bundleIdentifier ?? "me.sjun.inputmethod.Bada", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("state.json")
    }()

    /// The engine's state last reported, and the app it was for: kept so a Caps Lock or input
    /// source change can publish without an engine call.
    private static var engine: [String: Any] = [:]
    private static var app: String?
    private static var published: String?
    private static var watching = false

    /// After an engine call (or a mode, profile or focus change): the engine's state, then publish.
    static func publish(_ engineState: String?, app: String?) {
        if let engineState, let data = engineState.data(using: .utf8),
           let object = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] {
            engine = object
        }
        if let app { Self.app = app }
        refresh()
    }

    /// Publishes again with the host's fields as they are now (Caps Lock, input source).
    static func refresh() {
        watch()
        guard !engine.isEmpty else { return }
        var state = engine
        let secure = IsSecureEventInputEnabled()
        if secure {
            state["staged"] = ""
            state["preedit"] = ""
            state["menu"] = [String]()
        }
        let source = TISCopyCurrentKeyboardInputSource().takeRetainedValue()
        let sourceID = TISGetInputSourceProperty(source, kTISPropertyInputSourceID)
            .map { Unmanaged<CFString>.fromOpaque($0).takeUnretainedValue() as String }
        state["host"] = [
            "app": app as Any,
            "secure": secure,
            // The input source selected now: one of bada ime's two modes, or another.
            "input_source": sourceID as Any,
            "active": Modes.selectedNow() != nil,
            "caps_lock": [
                "on": CapsLock.isOn(NSEvent.modifierFlags),
                "handled": CapsLock.isHandled,
                "setting": CapsLock.setting.rawValue,
                "rule": CapsLock.rule.rawValue,
            ] as [String: Any],
            "version": Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "",
        ] as [String: Any]
        guard let data = try? JSONSerialization.data(withJSONObject: state, options: [.sortedKeys]),
              let json = String(data: data, encoding: .utf8), json != published else { return }
        published = json
        try? data.write(to: url, options: .atomic)
        DistributedNotificationCenter.default().postNotificationName(notification, object: json, userInfo: nil,
                                                                     deliverImmediately: true)
    }

    /// Input source changes (to another input method and back) publish too.
    private static func watch() {
        guard !watching else { return }
        watching = true
        DistributedNotificationCenter.default().addObserver(
            forName: Notification.Name(kTISNotifySelectedKeyboardInputSourceChanged as String), object: nil, queue: .main
        ) { _ in refresh() }
    }
}
