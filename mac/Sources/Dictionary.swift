import Cocoa

/// The user dictionary (docs/dictionary-contract.md): `dictionary.json` in bada ime's folder,
/// loaded at launch and again whenever bada is activated after the file changed.
enum UserDictionary {
    static var folder: URL {
        let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("bada-ime", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }
    static var url: URL { folder.appendingPathComponent("dictionary.json") }

    private static var loadedStamp: Date?
    /// What the last load found, for the settings window: "123 positive, 4 negative, 5 shortcuts".
    private(set) static var status = "No dictionary.json yet."

    /// Loads the file if it changed since the last load (or was removed).
    static func refresh() {
        let stamp = (try? FileManager.default.attributesOfItem(atPath: url.path))?[.modificationDate] as? Date
        guard stamp != loadedStamp else { return }
        loadedStamp = stamp
        let json = stamp == nil ? "" : (try? String(contentsOf: url, encoding: .utf8)) ?? ""
        guard let reply = bada_dictionary_load(json) else { return }
        defer { bada_string_free(reply) }
        let summary = (try? JSONSerialization.jsonObject(with: Data(String(cString: reply).utf8))) as? [String: Any] ?? [:]
        if stamp == nil {
            status = "No dictionary.json yet."
        } else if summary["ok"] as? Bool == true {
            status = "Loaded: \(summary["positive"] ?? 0) positive, \(summary["negative"] ?? 0) negative, \(summary["shortcuts"] ?? 0) shortcuts."
        } else {
            status = "dictionary.json wasn't loaded: \(summary["error"] ?? "unknown error")"
        }
        DiagnosticLog.note("dictionary: \(status)")
    }

    static func openFolder() {
        NSWorkspace.shared.open(folder)
    }
}
