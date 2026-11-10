import Carbon.HIToolbox
import Cocoa
import UniformTypeIdentifiers

/// The settings window (spec §5.3, §6.4, §6.5): henkan candidates, the default muhenkan layout, the
/// method allowed to add negative menu entries, and the F6–F10 bindings. Changes apply at once.
final class SettingsWindow: NSObject, NSWindowDelegate, NSTabViewDelegate {
    /// The tab shown last, kept when the window is rebuilt (import).
    private var selectedTab: String?

    func tabView(_ tabView: NSTabView, didSelect tabViewItem: NSTabViewItem?) {
        selectedTab = tabViewItem?.label
    }

    static let shared = SettingsWindow()

    private var window: NSWindow?
    private var layouts: [(id: UInt32, name: String, henkan: Bool)] = []
    private var actions: [(method: UInt32, action: String, label: String)] = []
    private var candidateBoxes: [NSButton] = []
    private var weightFields: [NSTextField] = []
    private var tonesField = NSTextField()
    private var muhenkanPopup = NSPopUpButton()
    private var negativePopup = NSPopUpButton()
    private var scriptFormsBox = NSButton()
    private var interceptBox = NSButton()
    private var fProfilePopup = NSPopUpButton()
    private var fOwnPopup = NSPopUpButton()
    /// The profile whose F keys the F Keys page shows and edits.
    private var fEditing = BadaProfiles.macKana
    private var fPopups: [NSPopUpButton] = []
    private var profilesBox = NSButton()
    private var linkPopups: [String: NSPopUpButton] = [:]
    private var keyLabels: [String: NSTextField] = [:]
    private var recordButtons: [String: NSButton] = [:]
    private var clearButtons: [String: NSButton] = [:]
    private var editPopup = NSPopUpButton()
    private var kanaPopups: [String: NSPopUpButton] = [:]
    private var orderPopup = NSPopUpButton()
    /// Every order of the three profiles, for the cycle order.
    private static let orders: [[String]] = {
        let names = BadaProfiles.all.map(\.name)
        return [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]].map { $0.map { names[$0] } }
    }()
    /// The profile whose own candidates the checkboxes show and edit.
    private var editing = BadaProfiles.macKana
    private var recording: String?
    private var keyMonitor: Any?

    func show(catalog: [String: Any]) {
        layouts = (catalog["layouts"] as? [[String: Any]] ?? []).compactMap { l in
            guard let id = l["id"] as? Int, let name = l["name"] as? String else { return nil }
            return (UInt32(id), name, l["henkan"] as? Bool ?? false)
        }
        actions = (catalog["actions"] as? [[String: Any]] ?? []).compactMap { a in
            guard let m = a["method"] as? Int, let action = a["action"] as? String, let label = a["label"] as? String else { return nil }
            return (UInt32(m), action, label)
        }
        build()
        // bada ime is background-only (LSBackgroundOnly): such an app can't be active, so its window
        // would never take keystrokes (text fields, recording a key). While Settings is open it's an
        // accessory app: active and key, still no Dock icon.
        NSApp.setActivationPolicy(.accessory)
        NSApp.activate(ignoringOtherApps: true)
        window?.makeKeyAndOrderFront(nil)
    }

    private func name(_ id: UInt32) -> String {
        layouts.first { $0.id == id }?.name ?? "Layout \(id)"
    }

    /// A label: a heading (`bold`), or text that belongs to a control.
    private func label(_ text: String, bold: Bool = false) -> NSTextField {
        let field = NSTextField(labelWithString: text)
        if bold {
            field.font = .boldSystemFont(ofSize: NSFont.systemFontSize + 1)
            field.setAccessibilityRole(.staticText)
            field.setAccessibilityLabel(text + ", heading")
        }
        return field
    }

    /// Explanation under a control: smaller, secondary, wrapping to the page's width.
    private func help(_ text: String) -> NSTextField {
        let field = NSTextField(wrappingLabelWithString: text)
        field.font = .systemFont(ofSize: NSFont.smallSystemFontSize)
        field.textColor = .secondaryLabelColor
        field.preferredMaxLayoutWidth = 600
        return field
    }

    /// Pages of the window, in the order they were made; shown as tabs in `order`.
    private var pages: [String: NSStackView] = [:]
    private static let order = ["Layouts", "Profiles", "Menu", "F Keys", "Dictionary", "Backup"]

    private func page(_ title: String) -> NSStackView {
        let s = NSStackView()
        s.orientation = .vertical
        s.alignment = .leading
        s.spacing = 10
        s.edgeInsets = NSEdgeInsets(top: 16, left: 20, bottom: 20, right: 20)
        pages[title] = s
        return s
    }

    /// A tab: its page in a scroll view, so nothing is out of reach on a small screen.
    private func tab(_ title: String) -> NSTabViewItem {
        let item = NSTabViewItem(identifier: title)
        item.label = title
        let scroll = NSScrollView()
        scroll.hasVerticalScroller = true
        scroll.drawsBackground = false
        let page = pages[title] ?? NSStackView()
        let clip = FlippedView()
        clip.translatesAutoresizingMaskIntoConstraints = false
        page.translatesAutoresizingMaskIntoConstraints = false
        clip.addSubview(page)
        NSLayoutConstraint.activate([
            page.topAnchor.constraint(equalTo: clip.topAnchor), page.leadingAnchor.constraint(equalTo: clip.leadingAnchor),
            page.trailingAnchor.constraint(equalTo: clip.trailingAnchor), page.bottomAnchor.constraint(equalTo: clip.bottomAnchor),
        ])
        scroll.documentView = clip
        clip.widthAnchor.constraint(equalTo: scroll.contentView.widthAnchor).isActive = true
        item.view = scroll
        return item
    }

    private func build() {
        let settings = BadaSettings.current
        pages = [:]
        var stack = page("Profiles")

        stack.addArrangedSubview(label("Henkan profiles", bold: true))
        profilesBox = NSButton(checkboxWithTitle: "変換 and カタカナ select their own henkan candidates (needs Accessibility, like 英数 and かな)",
                               target: self, action: #selector(changed(_:)))
        profilesBox.state = settings.use_profiles ? .on : .off
        stack.addArrangedSubview(profilesBox)
        for (name, key) in BadaProfiles.all {
            let row = NSStackView()
            row.orientation = .horizontal
            row.addArrangedSubview(label("\(key)  (profile-\(name))"))
            let popup = NSPopUpButton(frame: .zero, pullsDown: false)
            popup.addItem(withTitle: "Its own candidates")
            popup.lastItem?.representedObject = ""
            for (other, otherKey) in BadaProfiles.all where other != name {
                popup.addItem(withTitle: "Same as \(otherKey)")
                popup.lastItem?.representedObject = other
            }
            popup.target = self
            popup.action = #selector(linkChanged(_:))
            popup.identifier = NSUserInterfaceItemIdentifier(name)
            popup.setAccessibilityLabel("Candidates of the \(key) profile")
            row.addArrangedSubview(popup)
            linkPopups[name] = popup
            if name != BadaProfiles.macKana {
                let keyLabel = label("")
                keyLabels[name] = keyLabel
                row.addArrangedSubview(keyLabel)
                let button = NSButton(title: "Record Key…", target: self, action: #selector(recordKey(_:)))
                button.identifier = NSUserInterfaceItemIdentifier(name)
                button.setAccessibilityLabel("Record the \(key) key")
                recordButtons[name] = button
                row.addArrangedSubview(button)
                let clear = NSButton(title: "Clear", target: self, action: #selector(clearKey(_:)))
                clear.identifier = NSUserInterfaceItemIdentifier(name)
                clear.setAccessibilityLabel("Clear the \(key) key")
                clearButtons[name] = clear
                row.addArrangedSubview(clear)
            }
            stack.addArrangedSubview(row)
            if name != BadaProfiles.macKana {
                let kanaRow = NSStackView()
                kanaRow.orientation = .horizontal
                kanaRow.addArrangedSubview(label("      かな in this profile"))
                let popup = NSPopUpButton(frame: .zero, pullsDown: false)
                for option in KanaInProfile.allCases {
                    popup.addItem(withTitle: option.title)
                    popup.lastItem?.representedObject = option.rawValue
                }
                popup.target = self
                popup.action = #selector(changed(_:))
                popup.setAccessibilityLabel("What かな does in the \(key) profile")
                kanaRow.addArrangedSubview(popup)
                kanaPopups[name] = popup
                stack.addArrangedSubview(kanaRow)
            }
        }
        stack.addArrangedSubview(help("macOS may not pass a PC keyboard's 変換 and カタカナ keys on: record whatever key your keyboard sends for them."))
        let orderRow = NSStackView()
        orderRow.orientation = .horizontal
        orderRow.addArrangedSubview(label("Cycle order"))
        orderPopup = NSPopUpButton(frame: .zero, pullsDown: false)
        let keyOf = Dictionary(uniqueKeysWithValues: BadaProfiles.all.map { ($0.name, $0.key) })
        for (i, order) in Self.orders.enumerated() {
            orderPopup.addItem(withTitle: order.map { keyOf[$0] ?? $0 }.joined(separator: " → "))
            orderPopup.lastItem?.tag = i
        }
        orderPopup.target = self
        orderPopup.action = #selector(changed(_:))
        orderPopup.setAccessibilityLabel("Profile cycle order")
        orderRow.addArrangedSubview(orderPopup)
        stack.addArrangedSubview(orderRow)

        stack = page("Layouts")
        stack.addArrangedSubview(label("Henkan candidates", bold: true))
        stack.addArrangedSubview(help("Every enabled layout reads the keys at once; the readings are ranked as you type."))
        let editRow = NSStackView()
        editRow.orientation = .horizontal
        editRow.addArrangedSubview(label("Candidates of"))
        editPopup = NSPopUpButton(frame: .zero, pullsDown: false)
        for (name, key) in BadaProfiles.all {
            editPopup.addItem(withTitle: "\(key)  (profile-\(name))")
            editPopup.lastItem?.representedObject = name
        }
        editPopup.target = self
        editPopup.action = #selector(editingChanged(_:))
        editPopup.setAccessibilityLabel("Profile whose candidates the checkboxes below edit")
        editRow.addArrangedSubview(editPopup)
        stack.addArrangedSubview(editRow)
        stack.addArrangedSubview(help("× weight: multiplies the score of everything that layout offers, in every profile (1: as is, 0–10)."))
        weightFields = []
        candidateBoxes = layouts.map { layout in
            let row = NSStackView()
            row.orientation = .horizontal
            let box = NSButton(checkboxWithTitle: "\(layout.name)  (\(layout.id))", target: self, action: #selector(changed(_:)))
            box.tag = Int(layout.id)
            row.addArrangedSubview(box)
            row.addArrangedSubview(label("×"))
            let field = NSTextField(string: Self.weightText(settings.weights[String(layout.id)]))
            field.tag = Int(layout.id)
            field.alignment = .right
            field.widthAnchor.constraint(equalToConstant: 48).isActive = true
            field.target = self
            field.action = #selector(changed(_:))
            field.cell?.sendsActionOnEndEditing = true   // Tab or a click away saves it too
            field.setAccessibilityLabel("Weight for \(layout.name)")
            field.toolTip = "Multiplies the score of everything \(layout.name) offers (1: as is, 0 to 10)"
            row.addArrangedSubview(field)
            weightFields.append(field)
            stack.addArrangedSubview(row)
            return box
        }
        refreshProfiles(settings)

        stack.addArrangedSubview(label("Default muhenkan layout (英数)", bold: true))
        muhenkanPopup = NSPopUpButton(frame: .zero, pullsDown: false)
        for layout in layouts where !layout.henkan {
            muhenkanPopup.addItem(withTitle: "\(layout.name)  (\(layout.id))")
            muhenkanPopup.lastItem?.tag = Int(layout.id)
        }
        muhenkanPopup.selectItem(withTag: Int(settings.default_muhenkan))
        muhenkanPopup.target = self
        muhenkanPopup.action = #selector(changed(_:))
        stack.addArrangedSubview(muhenkanPopup)

        stack = page("Menu")
        stack.addArrangedSubview(label("Emoji skin tones (Symbols, 150)", bold: true))
        tonesField = NSTextField(string: settings.skin_tones.joined(separator: ", "))
        tonesField.widthAnchor.constraint(equalToConstant: 420).isActive = true
        tonesField.target = self
        tonesField.action = #selector(changed(_:))
        tonesField.cell?.sendsActionOnEndEditing = true
        tonesField.setAccessibilityLabel("Emoji skin tone order")
        stack.addArrangedSubview(tonesField)
        stack.addArrangedSubview(help("The order emoji with skin tones are offered in: none, light, medium-light, medium, medium-dark, dark. Leave one out to hide it."))

        stack.addArrangedSubview(label("Negative menu entries (⇧Tab from the top)", bold: true))
        scriptFormsBox = NSButton(checkboxWithTitle: "Script forms below the top entry: −1 hiragana, −2 katakana (off: katakana is a ranked entry)", target: self, action: #selector(changed(_:)))
        scriptFormsBox.state = settings.script_forms ? .on : .off
        stack.addArrangedSubview(scriptFormsBox)
        negativePopup = NSPopUpButton(frame: .zero, pullsDown: false)
        negativePopup.addItem(withTitle: "The lowest enabled method that offers them")
        negativePopup.lastItem?.tag = -1
        for layout in layouts where layout.henkan {
            negativePopup.addItem(withTitle: "\(layout.name)  (\(layout.id))")
            negativePopup.lastItem?.tag = Int(layout.id)
        }
        negativePopup.selectItem(withTag: settings.negative_method.map { Int($0) } ?? -1)
        negativePopup.target = self
        negativePopup.action = #selector(changed(_:))
        negativePopup.isEnabled = settings.script_forms
        stack.addArrangedSubview(negativePopup)
        stack.addArrangedSubview(help("Japanese (Kana) and Japanese (Romaji) offer them: −1 hiragana, −2 katakana."))

        stack = page("F Keys")
        stack.addArrangedSubview(label("F6–F10", bold: true))
        let fRow = NSStackView()
        fRow.orientation = .horizontal
        fRow.addArrangedSubview(label("F keys of"))
        fProfilePopup = NSPopUpButton(frame: .zero, pullsDown: false)
        for (name, key) in BadaProfiles.all {
            fProfilePopup.addItem(withTitle: "\(key)  (profile-\(name))")
            fProfilePopup.lastItem?.representedObject = name
        }
        fProfilePopup.setAccessibilityLabel("Profile whose F keys this page edits")
        fProfilePopup.target = self
        fProfilePopup.action = #selector(fProfileChanged(_:))
        fRow.addArrangedSubview(fProfilePopup)
        fOwnPopup = NSPopUpButton(frame: .zero, pullsDown: false)
        fOwnPopup.addItem(withTitle: "Same as かな's F keys")
        fOwnPopup.addItem(withTitle: "Its own F keys")
        fOwnPopup.setAccessibilityLabel("Whether this profile has its own F keys")
        fOwnPopup.target = self
        fOwnPopup.action = #selector(fOwnChanged(_:))
        fRow.addArrangedSubview(fOwnPopup)
        stack.addArrangedSubview(fRow)
        stack.addArrangedSubview(help("Each henkan profile can bind F6–F10 its own way, or turn them off, while it's in use. Profiles need Henkan profiles on (Profiles tab)."))
        interceptBox = NSButton(checkboxWithTitle: "Intercept F6–F10 (off: every F key goes to the app)", target: self, action: #selector(changed(_:)))
        interceptBox.state = settings.intercept_f ? .on : .off
        stack.addArrangedSubview(interceptBox)
        fPopups = (6...10).map { f in
            let row = NSStackView()
            row.orientation = .horizontal
            row.addArrangedSubview(label("F\(f)"))
            let popup = NSPopUpButton(frame: .zero, pullsDown: false)
            popup.setAccessibilityLabel("Action of F\(f)")
            popup.addItem(withTitle: "Swallowed (does nothing)")
            popup.lastItem?.representedObject = "swallow"
            popup.addItem(withTitle: "Passed through to the app")
            popup.lastItem?.representedObject = "pass"
            for action in actions {
                popup.addItem(withTitle: "\(name(action.method)): \(action.label)")
                popup.lastItem?.representedObject = "\(action.method).\(action.action)"
            }
            let current = settings.fkeys["F\(f)"] ?? "swallow"
            if let item = popup.itemArray.first(where: { $0.representedObject as? String == current }) {
                popup.select(item)
            }
            popup.target = self
            popup.action = #selector(changed(_:))
            row.addArrangedSubview(popup)
            stack.addArrangedSubview(row)
            return popup
        }
        stack.addArrangedSubview(help("⇧, ⌥ and ⌃ with an F key: up to the bound method (bada defines none, so they're swallowed)."))
        refreshFKeys(settings)

        stack = page("Dictionary")
        stack.addArrangedSubview(label("Your dictionary", bold: true))
        UserDictionary.refresh()
        stack.addArrangedSubview(label(UserDictionary.status))
        stack.addArrangedSubview(help("Positive and negative words: docs/dictionary-contract.md. It reloads when the file changes."))
        stack.addArrangedSubview(NSButton(title: "Open Dictionary Folder", target: self, action: #selector(openDictionary(_:))))

        stack = page("Backup")
        stack.addArrangedSubview(label("Settings file", bold: true))
        let fileRow = NSStackView()
        fileRow.orientation = .horizontal
        fileRow.addArrangedSubview(NSButton(title: "Export Settings…", target: self, action: #selector(exportSettings(_:))))
        fileRow.addArrangedSubview(NSButton(title: "Import Settings…", target: self, action: #selector(importSettings(_:))))
        stack.addArrangedSubview(fileRow)
        stack.addArrangedSubview(help("Everything on this page, as JSON (bada-settings/1). Importing replaces it all."))

        let tabs = NSTabView()
        for title in Self.order where pages[title] != nil {
            tabs.addTabViewItem(tab(title))
        }
        if let selected = selectedTab, let item = tabs.tabViewItems.first(where: { $0.label == selected }) {
            tabs.selectTabViewItem(item)
        }
        tabs.delegate = self
        let window = self.window ?? NSWindow(contentRect: NSRect(x: 0, y: 0, width: 700, height: 620),
                                             styleMask: [.titled, .closable, .resizable], backing: .buffered, defer: false)
        window.title = "bada ime Settings"
        window.contentView = tabs
        window.isReleasedWhenClosed = false
        window.delegate = self
        window.contentMinSize = NSSize(width: 560, height: 360)
        window.autorecalculatesKeyViewLoop = true   // Tab moves through every control
        if self.window == nil {
            window.setContentSize(NSSize(width: 700, height: 620))
            window.center()
        }
        self.window = window
    }

    private static func weightText(_ weight: Double?) -> String {
        let w = weight ?? 1
        return w == w.rounded() ? String(Int(w)) : String(w)
    }

    // MARK: F keys per profile (spec §6.5, §6.6)

    /// The F keys in effect for `name`: its own, or かな's.
    private static func fKeys(_ settings: BadaSettings, _ name: String) -> (own: Bool, keys: BadaFKeys) {
        if name != BadaProfiles.macKana, let own = settings.profiles[name]?.f_keys { return (true, own) }
        return (false, BadaFKeys(intercept: settings.intercept_f, keys: settings.fkeys))
    }

    private func refreshFKeys(_ settings: BadaSettings) {
        if !settings.use_profiles { fEditing = BadaProfiles.macKana }
        if let item = fProfilePopup.itemArray.first(where: { $0.representedObject as? String == fEditing }) { fProfilePopup.select(item) }
        fProfilePopup.isEnabled = settings.use_profiles
        let (own, keys) = Self.fKeys(settings, fEditing)
        let isKana = fEditing == BadaProfiles.macKana
        fOwnPopup.selectItem(at: own ? 1 : 0)
        fOwnPopup.isHidden = isKana
        let editable = isKana || own
        interceptBox.state = keys.intercept ? .on : .off
        interceptBox.isEnabled = editable
        for (i, popup) in fPopups.enumerated() {
            let current = keys.keys["F\(i + 6)"] ?? "swallow"
            if let item = popup.itemArray.first(where: { $0.representedObject as? String == current }) { popup.select(item) }
            popup.isEnabled = editable && keys.intercept
        }
    }

    @objc private func fProfileChanged(_ sender: NSPopUpButton) {
        fEditing = sender.selectedItem?.representedObject as? String ?? BadaProfiles.macKana
        refreshFKeys(BadaSettings.current)
    }

    @objc private func fOwnChanged(_ sender: NSPopUpButton) {
        guard fEditing != BadaProfiles.macKana else { return }
        var settings = BadaSettings.current
        var profile = settings.profiles[fEditing] ?? BadaProfile(link: BadaProfiles.macKana)
        // Its own: starts as a copy of かな's. Same as かな's: its own are dropped.
        profile.f_keys = sender.indexOfSelectedItem == 1 ? BadaFKeys(intercept: settings.intercept_f, keys: settings.fkeys) : nil
        settings.profiles[fEditing] = profile
        BadaSettings.current = settings
        refreshFKeys(settings)
    }

    // MARK: henkan profiles (spec §6.6)

    private static func link(_ settings: BadaSettings, _ name: String) -> String? {
        if let p = settings.profiles[name] { return p.link }
        return name == BadaProfiles.macKana ? nil : BadaProfiles.macKana
    }

    private static func own(_ settings: BadaSettings, _ name: String) -> [UInt32] {
        name == BadaProfiles.macKana ? settings.enabled : settings.profiles[name]?.enabled ?? []
    }

    private func keyTitle(_ code: Int?) -> String {
        guard let code else { return "Key: none" }
        if let hid = HIDKeys.name(code) { return "Key: \(hid)" }
        let name = keyName(UInt16(code))
        return "Key: \(name == "Other" ? "code \(code)" : name)"
    }

    /// Shows the profiles' links, keys and the edited profile's candidates.
    private func refreshProfiles(_ settings: BadaSettings) {
        let on = settings.use_profiles
        if !on { editing = BadaProfiles.macKana }
        for (name, popup) in linkPopups {
            let link = Self.link(settings, name) ?? ""
            if let item = popup.itemArray.first(where: { $0.representedObject as? String == link }) { popup.select(item) }
            popup.isEnabled = on
        }
        for (name, field) in keyLabels { field.stringValue = keyTitle(settings.profile_keys[name]) }
        for (name, button) in clearButtons {
            button.isEnabled = on && settings.profile_keys[name] != nil && recording == nil
        }
        for (name, button) in recordButtons {
            button.isEnabled = on
            button.title = recording == name ? "Press a Key (esc: Cancel)" : "Record Key…"
        }
        for (name, popup) in kanaPopups {
            let value = KanaInProfile.of(settings, name).rawValue
            if let item = popup.itemArray.first(where: { $0.representedObject as? String == value }) { popup.select(item) }
            popup.isEnabled = on
        }
        orderPopup.selectItem(withTag: Self.orders.firstIndex(of: settings.profile_order) ?? 0)
        orderPopup.isEnabled = on && kanaPopups.keys.contains { KanaInProfile.of(settings, $0) == .cycle }
        if let item = editPopup.itemArray.first(where: { $0.representedObject as? String == editing }) { editPopup.select(item) }
        editPopup.isEnabled = on
        let linked = Self.link(settings, editing) != nil
        let list = Self.own(settings, editing)
        for box in candidateBoxes {
            box.state = list.contains(UInt32(box.tag)) ? .on : .off
            box.isEnabled = !linked
        }
    }

    @objc private func linkChanged(_ sender: NSPopUpButton) {
        guard let name = sender.identifier?.rawValue else { return }
        var settings = BadaSettings.current
        let link = (sender.selectedItem?.representedObject as? String).flatMap { $0.isEmpty ? nil : $0 }
        var profile = settings.profiles[name] ?? BadaProfile()
        if link == nil && Self.link(settings, name) != nil && Self.own(settings, name).isEmpty && name != BadaProfiles.macKana {
            profile.enabled = settings.enabled   // a new own list starts as かな's
        }
        profile.link = link
        settings.profiles[name] = profile
        editing = name
        BadaSettings.current = settings
        refreshProfiles(settings)
    }

    @objc private func editingChanged(_ sender: NSPopUpButton) {
        editing = sender.selectedItem?.representedObject as? String ?? BadaProfiles.macKana
        refreshProfiles(BadaSettings.current)
    }

    /// Records the next key for a profile: through the key tap (it sees every key, wherever focus
    /// is), with this window's own key events as a fallback. Esc cancels; かな and 英数 are refused.
    @objc private func recordKey(_ sender: NSButton) {
        guard let name = sender.identifier?.rawValue else { return }
        if recording == name {   // clicked again: cancel
            stopRecording()
            return
        }
        recording = name
        refreshProfiles(BadaSettings.current)
        LanguageKeyTap.start(ask: true)
        HIDKeys.start(ask: true)
        LanguageKeyTap.recorder = { [weak self] code in self?.recorded(code) }
        HIDKeys.recorder = { [weak self] code in self?.recorded(code) }
        if let keyMonitor { NSEvent.removeMonitor(keyMonitor) }
        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self, self.recording != nil else { return event }
            self.recorded(Int(event.keyCode))
            return nil
        }
    }

    /// Forgets a profile key: the profile is then reachable only through links (spec §6.6).
    @objc private func clearKey(_ sender: NSButton) {
        guard let name = sender.identifier?.rawValue else { return }
        var settings = BadaSettings.current
        settings.profile_keys.removeValue(forKey: name)
        BadaSettings.current = settings
        refreshProfiles(settings)
    }

    private func recorded(_ code: Int) {
        guard let name = recording else { return }
        if code == kVK_JIS_Kana || code == kVK_JIS_Eisu {
            // かな and 英数 keep their own jobs: wait for another key. (カタカナ also arrives as かな;
            // its HID usage, read at the same moment, is what gets recorded.)
            if LanguageKeyTap.isOn { LanguageKeyTap.recorder = { [weak self] code in self?.recorded(code) } }
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
                if self?.recording == name { NSSound.beep() }
            }
            return
        }
        if code != kVK_Escape {
            var settings = BadaSettings.current
            settings.profile_keys[name] = code
            BadaSettings.current = settings
        }
        stopRecording()
    }

    private func stopRecording() {
        recording = nil
        LanguageKeyTap.recorder = nil
        HIDKeys.recorder = nil
        if let keyMonitor { NSEvent.removeMonitor(keyMonitor) }
        keyMonitor = nil
        refreshProfiles(BadaSettings.current)
    }

    func windowWillClose(_ notification: Notification) {
        if recording != nil { stopRecording() }
        window?.makeFirstResponder(nil)   // ends editing: a typed weight is saved
        DispatchQueue.main.async { NSApp.setActivationPolicy(.prohibited) }
    }

    // MARK: export and import

    @objc private func exportSettings(_ sender: Any?) {
        let panel = NSSavePanel()
        panel.nameFieldStringValue = "bada-settings.json"
        panel.allowedContentTypes = [.json]
        NSApp.activate(ignoringOtherApps: true)
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try BadaSettings.current.exported().write(to: url)
        } catch {
            alert("Couldn't export the settings", error.localizedDescription)
        }
    }

    @objc private func importSettings(_ sender: Any?) {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.allowsMultipleSelection = false
        NSApp.activate(ignoringOtherApps: true)
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            BadaSettings.current = try BadaSettings.imported(Data(contentsOf: url))
            editing = BadaProfiles.macKana
            build()
            window?.makeKeyAndOrderFront(nil)
        } catch {
            alert("Couldn't import the settings", error.localizedDescription)
        }
    }

    private func alert(_ title: String, _ text: String) {
        let a = NSAlert()
        a.messageText = title
        a.informativeText = text
        a.runModal()
    }

    @objc private func changed(_ sender: Any?) {
        var settings = BadaSettings.current
        settings.use_profiles = profilesBox.state == .on
        let tones = tonesField.stringValue.lowercased().split(whereSeparator: { $0 == "," || $0 == " " })
            .map(String.init).filter { BadaSettings.allTones.contains($0) }
        var seen = Set<String>()
        settings.skin_tones = tones.filter { seen.insert($0).inserted }
        tonesField.stringValue = settings.skin_tones.joined(separator: ", ")
        for field in weightFields {
            let id = String(field.tag)
            let text = field.stringValue.trimmingCharacters(in: .whitespaces).replacingOccurrences(of: ",", with: ".")
            if let w = Double(text), w.isFinite {
                let clamped = min(max(w, 0), 10)
                if clamped == 1 { settings.weights.removeValue(forKey: id) } else { settings.weights[id] = clamped }
                field.stringValue = Self.weightText(clamped)
            } else {
                field.stringValue = Self.weightText(settings.weights[id])   // not a number: back to what it was
            }
        }
        for (name, popup) in kanaPopups {
            settings.kana_in_profile[name] = popup.selectedItem?.representedObject as? String ?? KanaInProfile.nextOrBack.rawValue
        }
        if let i = orderPopup.selectedItem?.tag, Self.orders.indices.contains(i) { settings.profile_order = Self.orders[i] }
        if Self.link(settings, editing) == nil {
            let list = candidateBoxes.filter { $0.state == .on }.map { UInt32($0.tag) }
            if editing == BadaProfiles.macKana {
                settings.enabled = list
            } else {
                settings.profiles[editing, default: BadaProfile()].enabled = list
            }
        }
        if let tag = muhenkanPopup.selectedItem?.tag { settings.default_muhenkan = UInt32(tag) }
        settings.script_forms = scriptFormsBox.state == .on
        negativePopup.isEnabled = settings.script_forms
        let negative = negativePopup.selectedItem?.tag ?? -1
        settings.negative_method = negative < 0 ? nil : UInt32(negative)
        // F keys: into かな's (the settings' own) or the edited profile's, if it has its own.
        var f = BadaFKeys(intercept: interceptBox.state == .on, keys: [:])
        for (i, popup) in fPopups.enumerated() {
            let value = popup.selectedItem?.representedObject as? String ?? "swallow"
            if value != "swallow" { f.keys["F\(i + 6)"] = value }
        }
        if fEditing == BadaProfiles.macKana {
            settings.intercept_f = f.intercept
            settings.fkeys = f.keys
        } else if settings.profiles[fEditing]?.f_keys != nil {
            settings.profiles[fEditing]?.f_keys = f
        }
        BadaSettings.current = settings
        refreshProfiles(settings)
        refreshFKeys(settings)
    }

    @objc private func openDictionary(_ sender: Any?) {
        UserDictionary.openFolder()
    }
}

/// A document view that lays out top-down, so a scrolled page starts at its top.
private final class FlippedView: NSView {
    override var isFlipped: Bool { true }
}

