import Cocoa

/// 초성 diagnostics (spec §8): the engine counts, per initial key, the vowels and finals attached
/// to it (henkan mode only, counts only). This keeps them in a file and saves them now and then.
enum ChoseongDiagnostics {
    static var url: URL {
        let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("bada-ime", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("choseong-diagnostics.json")
    }

    private static var timer: Timer?

    /// At launch: the saved counts, the setting, and a save every 30 seconds when something changed.
    static func start() {
        if let json = try? String(contentsOf: url, encoding: .utf8) {
            bada_diag_import(json)
        }
        apply()
        timer = Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { _ in save() }
    }

    static func apply() {
        bada_diag_set_enabled(BadaSettings.current.collect_choseong ? 1 : 0)
    }

    static func save() {
        guard bada_diag_changes() > 0, let text = bada_diag_export() else { return }
        defer { bada_string_free(text) }
        try? String(cString: text).write(to: url, atomically: true, encoding: .utf8)
    }

    static func clear() {
        bada_diag_clear()
        save()
    }

    static func reveal() {
        save()
        if !FileManager.default.fileExists(atPath: url.path) {
            try? "{\"format\":\"bada-choseong-diagnostics/1\",\"rows\":[]}".write(to: url, atomically: true, encoding: .utf8)
        }
        NSWorkspace.shared.activateFileViewerSelecting([url])
    }
}
