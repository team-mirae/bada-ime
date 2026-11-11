import Carbon.HIToolbox
import Cocoa
import InputMethodKit

/// One controller per text input client; each owns an engine. All behaviour is in the engine: this
/// class moves keys in and text out, switches input sources, and shows the menu.
@objc(BadaInputController)
final class BadaInputController: IMKInputController {
    private let engine = Engine()
    private var hasMarkedText = false
    /// Rows of the menu shown last: かな moves in it only when there's one (spec §6.6).
    private var menuRows = 0
    private var settingsObserver: NSObjectProtocol?

    /// The controller that got the last key, else the last activated one: where 英数 / かな act.
    private(set) static weak var focused: BadaInputController?
    private(set) static weak var keyed: BadaInputController?
    static var typing: BadaInputController? { keyed ?? focused }

    override init!(server: IMKServer!, delegate: Any!, client inputClient: Any!) {
        super.init(server: server, delegate: delegate, client: inputClient)
        engine.apply(BadaSettings.current.engineJSON)
        settingsObserver = NotificationCenter.default.addObserver(forName: BadaSettings.changed, object: nil, queue: .main) { [weak self] _ in
            self?.engine.apply(BadaSettings.current.engineJSON)
        }
    }

    deinit {
        if let settingsObserver { NotificationCenter.default.removeObserver(settingsObserver) }
    }

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask.keyDown.rawValue)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event, event.type == .keyDown, let client = sender as? IMKTextInput else { return false }
        Self.keyed = self
        let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let shift = flags.contains(.shift), ctrl = flags.contains(.control)
        let alt = flags.contains(.option), cmd = flags.contains(.command)
        let code = Int(event.keyCode)
        // The 英数 / かな the key tap posted for macOS's switch (LanguageKeys.go): acted on when the
        // mode is selected, not here.
        if (code == kVK_JIS_Eisu || code == kVK_JIS_Kana) && Date().timeIntervalSince(LanguageKeyTap.lastPosted) < 1 {
            return true
        }
        // 英数 and かな, when they reach the input method (without the key tap).
        if code == kVK_JIS_Eisu {
            if !event.isARepeat { switchMode(to: .muhenkan, client: client) }
            return true
        }
        var profileKey = LanguageKeyTap.profileKeys[code]
        if (code == kVK_JIS_Kana || code == kVK_JIS_Eisu) && !event.isARepeat, let hid = HIDKeys.claim() {
            if hid.recorded { return true }
            if let name = LanguageKeyTap.profileKeys[hid.code] { profileKey = name }   // カタカナ, as かな
        }
        if code == kVK_JIS_Kana || (profileKey != nil && !ctrl && !alt && !cmd) {
            if !event.isARepeat {
                if code == kVK_JIS_Kana && ctrl && alt && shift {
                    switchMode(to: Modes.current.other, client: client)
                } else if Modes.current == .henkan {
                    languageKey(profileKey ?? BadaProfiles.macKana, shift: shift, client: client)
                } else {
                    Modes.profile = BadaSettings.current.use_profiles ? profileKey ?? BadaProfiles.macKana : BadaProfiles.macKana
                    switchMode(to: .henkan, client: client)
                }
            }
            return true
        }
        // In a terminal, ⌥ in muhenkan goes to the app untouched (⌥ as Meta: readline's ⌥B, ⌥F).
        if alt && Modes.current == .muhenkan && Self.terminals.contains(client.bundleIdentifier() ?? "") {
            return false
        }
        let out = engine.key(keyName(event.keyCode), shift: shift, ctrl: ctrl, alt: alt, meta: cmd)
        apply(out, to: client)
        // In henkan the keyboard is bada.keylayout, for the Keyboard Viewer: a key passed on that
        // carries a jamo there is typed as ABC types it (as it would be in muhenkan).
        if !out.consumed && Modes.current == .henkan && !cmd && !ctrl && !alt,
           let text = Keyboards.passOn(code: code, shift: shift) {
            client.insertText(text, replacementRange: NSRange(location: NSNotFound, length: 0))
            return true
        }
        return out.consumed
    }

    private static let terminals: Set<String> = [
        "com.apple.Terminal", "com.googlecode.iterm2", "net.kovidgoyal.kitty", "com.github.wez.wezterm",
        "io.alacritty", "org.alacritty", "dev.warp.Warp-Stable", "com.mitchellh.ghostty",
    ]

    /// かな again, or ⇧かな, in henkan mode: the next or previous menu entry.
    func cycle(back: Bool, client given: IMKTextInput? = nil) {
        guard let client = given ?? (client() as IMKTextInput?) else { return }
        apply(engine.key("Lang1", shift: back, ctrl: false, alt: false, meta: false), to: client)
    }

    /// かな or a profile key pressed in henkan mode (spec §6.6). The same profile's key moves in the
    /// menu; another's switches profile. かな in another profile than mac-kana does what the
    /// setting says.
    func languageKey(_ key: String, shift: Bool, client given: IMKTextInput? = nil) {
        let settings = BadaSettings.current
        switch ProfileKeyLogic.action(key: key, current: Modes.profile, shift: shift, menu: menuRows > 1,
                                      profilesOn: settings.use_profiles, kanaIn: settings.kana_in_profile,
                                      order: settings.profile_order) {
        case .step(let back): cycle(back: back, client: given)
        case .profile(let name): switchProfile(to: name, client: given)
        case .muhenkan: switchMode(to: .muhenkan, client: given)
        }
    }

    /// To henkan profile `name` (spec §6.6): commits what's staged.
    func switchProfile(to name: String, client given: IMKTextInput? = nil) {
        Modes.profile = name
        guard let client = given ?? (client() as IMKTextInput?) else { return }
        apply(engine.setProfile(name), to: client)
    }

    /// To `mode`, in place: commits what's staged, then tells macOS.
    func switchMode(to mode: BadaMode, client given: IMKTextInput? = nil) {
        guard let client = given ?? (client() as IMKTextInput?) else {
            Modes.select(mode)
            return
        }
        apply(engine.setMode(mode), to: client)
        _ = engine.setProfile(Modes.profile)
        Modes.current = mode
        Keyboards.show(for: mode, in: client)
        DispatchQueue.main.async { Modes.select(mode, in: client) }
    }

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        Self.focused = self
        let client = sender as? IMKTextInput
        let mode = Modes.selectedNow() ?? Modes.current
        Modes.current = mode
        if let profile = LanguageKeys.selected(mode) { Modes.profile = profile }
        _ = engine.setMode(mode)
        engine.apply(BadaSettings.current.engineJSON)
        _ = engine.setProfile(Modes.profile)
        if let client { Keyboards.show(for: mode, in: client) }
        UserDictionary.refresh()
        DispatchQueue.main.async { LanguageKeyTap.check() }
    }

    override func setValue(_ value: Any!, forTag tag: Int, client sender: Any!) {
        if tag == kTextServiceInputModePropertyTag, let id = value as? String, let mode = Modes.mode(sourceID: id) {
            if let client = sender as? IMKTextInput {
                apply(engine.setMode(mode), to: client)
            }
            Modes.current = mode
            // macOS switched mode: if a language key asked for it, that key's profile.
            if let profile = LanguageKeys.selected(mode) { Modes.profile = profile }
            _ = engine.setProfile(Modes.profile)
        }
        super.setValue(value, forTag: tag, client: sender)
        // After super: selecting the mode puts its own keyboard in place.
        if tag == kTextServiceInputModePropertyTag, let client = sender as? IMKTextInput {
            Keyboards.show(for: Modes.current, in: client)
        }
    }

    override func deactivateServer(_ sender: Any!) {
        flush(sender)
        SuggestionWindow.shared.hide()
        super.deactivateServer(sender)
    }

    override func commitComposition(_ sender: Any!) {
        flush(sender)
    }

    private func flush(_ sender: Any!) {
        guard let client = (sender as? IMKTextInput) ?? (client() as IMKTextInput?) else { return }
        apply(engine.flush(), to: client)
    }

    // MARK: Output

    private func apply(_ out: EngineOutput, to client: IMKTextInput, depth: Int = 0) {
        let notFound = NSRange(location: NSNotFound, length: NSNotFound)
        if !out.commit.isEmpty {
            let range = out.replace_selection ? client.selectedRange() : notFound
            client.insertText(out.commit, replacementRange: range)
            hasMarkedText = false
        }
        if !out.preedit.isEmpty {
            let marked = NSAttributedString(string: out.preedit, attributes: [
                .underlineStyle: NSUnderlineStyle.single.rawValue, .markedClauseSegment: 0,
            ])
            client.setMarkedText(marked, selectionRange: NSRange(location: out.preedit.utf16.count, length: 0), replacementRange: notFound)
            hasMarkedText = true
        } else if hasMarkedText {
            client.setMarkedText("", selectionRange: NSRange(location: 0, length: 0), replacementRange: notFound)
            hasMarkedText = false
        }
        showMenu(out, client)
        menuRows = out.menu.count
        // The engine needs the selection (hanja, 半角/全角 with nothing staged).
        if out.need_selection != nil && depth == 0 {
            let selection = client.selectedRange()
            var actual = NSRange(location: NSNotFound, length: 0)
            let text = selection.location != NSNotFound && selection.length > 0
                ? client.string(from: selection, actualRange: &actual) ?? "" : ""
            apply(engine.hostSelection(text), to: client, depth: 1)
        }
    }

    private var menuGeneration = 0

    private func showMenu(_ out: EngineOutput, _ client: IMKTextInput) {
        menuGeneration += 1
        let generation = menuGeneration
        guard !out.menu.isEmpty else {
            SuggestionWindow.shared.hide()
            return
        }
        // The caret's rectangle is a round trip into the app: asked once the key is done.
        DispatchQueue.main.async { [weak self] in
            guard let self, generation == self.menuGeneration else { return }
            var caret = NSRect.zero
            _ = client.attributes(forCharacterIndex: 0, lineHeightRectangle: &caret)
            SuggestionWindow.shared.onPick = nil
            SuggestionWindow.shared.show(out.menu, selected: out.selected < out.menu.count ? out.selected : nil,
                                         below: caret, horizontal: out.horizontal, japanese: out.japanese, notes: out.notes)
        }
    }

    // MARK: Menu

    override func menu() -> NSMenu! {
        let menu = NSMenu()
        menu.addItem(NSMenuItem(title: "bada ime Settings…", action: #selector(openSettings(_:)), keyEquivalent: ""))
        menu.addItem(NSMenuItem(title: "Open Dictionary Folder", action: #selector(openDictionaryFolder(_:)), keyEquivalent: ""))
        if !AXIsProcessTrusted() {
            menu.addItem(NSMenuItem(title: "Allow 英数 and かな Everywhere…", action: #selector(allowKeys(_:)), keyEquivalent: ""))
        }
        return menu
    }

    @objc func openSettings(_ sender: Any?) {
        SettingsWindow.shared.show(catalog: engine.catalog())
    }

    @objc func openDictionaryFolder(_ sender: Any?) {
        UserDictionary.openFolder()
    }

    @objc func allowKeys(_ sender: Any?) {
        LanguageKeyTap.start(ask: true)
    }
}
