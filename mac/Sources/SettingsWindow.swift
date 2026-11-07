import Cocoa

/// The settings window (spec §5.3, §6.4, §6.5): henkan candidates, the default muhenkan layout, the
/// method allowed to add negative menu entries, and the F6–F10 bindings. Changes apply at once.
final class SettingsWindow: NSObject, NSWindowDelegate {
    static let shared = SettingsWindow()

    private var window: NSWindow?
    private var layouts: [(id: UInt32, name: String, henkan: Bool)] = []
    private var actions: [(method: UInt32, action: String, label: String)] = []
    private var candidateBoxes: [NSButton] = []
    private var muhenkanPopup = NSPopUpButton()
    private var negativePopup = NSPopUpButton()
    private var interceptBox = NSButton()
    private var fPopups: [NSPopUpButton] = []

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
        NSApp.activate(ignoringOtherApps: true)
        window?.makeKeyAndOrderFront(nil)
    }

    private func name(_ id: UInt32) -> String {
        layouts.first { $0.id == id }?.name ?? "Layout \(id)"
    }

    private func label(_ text: String, bold: Bool = false) -> NSTextField {
        let field = NSTextField(labelWithString: text)
        if bold { field.font = .boldSystemFont(ofSize: NSFont.systemFontSize) }
        return field
    }

    private func build() {
        let settings = BadaSettings.current
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 8
        stack.edgeInsets = NSEdgeInsets(top: 20, left: 20, bottom: 20, right: 20)

        stack.addArrangedSubview(label("Henkan candidates", bold: true))
        stack.addArrangedSubview(label("Every enabled layout reads the keys at once; the readings are ranked as you type."))
        candidateBoxes = layouts.map { layout in
            let box = NSButton(checkboxWithTitle: "\(layout.name)  (\(layout.id))", target: self, action: #selector(changed(_:)))
            box.tag = Int(layout.id)
            box.state = settings.enabled.contains(layout.id) ? .on : .off
            stack.addArrangedSubview(box)
            return box
        }

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

        stack.addArrangedSubview(label("Negative menu entries (⇧Tab from the top)", bold: true))
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
        stack.addArrangedSubview(negativePopup)
        stack.addArrangedSubview(label("Japanese (Kana) and Japanese (Romaji) offer them: −1 hiragana, −2 katakana."))

        stack.addArrangedSubview(label("F6–F10", bold: true))
        interceptBox = NSButton(checkboxWithTitle: "Intercept F6–F10 (off: every F key goes to the app)", target: self, action: #selector(changed(_:)))
        interceptBox.state = settings.intercept_f ? .on : .off
        stack.addArrangedSubview(interceptBox)
        fPopups = (6...10).map { f in
            let row = NSStackView()
            row.orientation = .horizontal
            row.addArrangedSubview(label("F\(f)"))
            let popup = NSPopUpButton(frame: .zero, pullsDown: false)
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
        stack.addArrangedSubview(label("⇧, ⌥ and ⌃ with an F key: up to the bound method (bada defines none, so they're swallowed)."))

        let window = self.window ?? NSWindow(contentRect: NSRect(x: 0, y: 0, width: 560, height: 600),
                                             styleMask: [.titled, .closable], backing: .buffered, defer: false)
        window.title = "bada ime Settings"
        window.contentView = stack
        window.isReleasedWhenClosed = false
        window.delegate = self
        window.setContentSize(stack.fittingSize)
        window.center()
        self.window = window
    }

    @objc private func changed(_ sender: Any?) {
        var settings = BadaSettings.current
        settings.enabled = candidateBoxes.filter { $0.state == .on }.map { UInt32($0.tag) }
        if let tag = muhenkanPopup.selectedItem?.tag { settings.default_muhenkan = UInt32(tag) }
        let negative = negativePopup.selectedItem?.tag ?? -1
        settings.negative_method = negative < 0 ? nil : UInt32(negative)
        settings.intercept_f = interceptBox.state == .on
        for (i, popup) in fPopups.enumerated() {
            let value = popup.selectedItem?.representedObject as? String ?? "swallow"
            if value == "swallow" { settings.fkeys.removeValue(forKey: "F\(i + 6)") } else { settings.fkeys["F\(i + 6)"] = value }
        }
        BadaSettings.current = settings
    }
}
