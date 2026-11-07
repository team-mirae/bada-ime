import Foundation

/// The user's settings (spec §5.3, §6.4, §6.5), stored as the engine's JSON in UserDefaults.
struct BadaSettings: Codable, Equatable {
    /// Henkan candidates (layout ids), in order.
    var enabled: [UInt32] = [13]
    var default_muhenkan: UInt32 = 0
    /// The one method allowed to add negative menu entries; nil: the lowest enabled that offers them.
    var negative_method: UInt32?
    var intercept_f = true
    /// "F6"…"F10" → "swallow" | "pass" | "<method>.<action>". Nothing is bound by default.
    var fkeys: [String: String] = [:]
    /// Collect 초성 diagnostics (spec §8): on by default, counts only, this Mac only.
    var collect_choseong = true

    enum CodingKeys: String, CodingKey {
        case enabled, default_muhenkan, negative_method, intercept_f, fkeys, collect_choseong
    }

    init() {}

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        enabled = try c.decodeIfPresent([UInt32].self, forKey: .enabled) ?? [13]
        default_muhenkan = try c.decodeIfPresent(UInt32.self, forKey: .default_muhenkan) ?? 0
        negative_method = try c.decodeIfPresent(UInt32.self, forKey: .negative_method)
        intercept_f = try c.decodeIfPresent(Bool.self, forKey: .intercept_f) ?? true
        fkeys = try c.decodeIfPresent([String: String].self, forKey: .fkeys) ?? [:]
        collect_choseong = try c.decodeIfPresent(Bool.self, forKey: .collect_choseong) ?? true
    }

    static let key = "Settings"
    static let changed = Notification.Name("BadaSettingsChanged")

    static var current: BadaSettings {
        get {
            guard let json = UserDefaults.standard.string(forKey: key), let data = json.data(using: .utf8),
                  let settings = try? JSONDecoder().decode(BadaSettings.self, from: data) else { return BadaSettings() }
            return settings
        }
        set {
            UserDefaults.standard.set(newValue.json, forKey: key)
            NotificationCenter.default.post(name: changed, object: nil)
            ChoseongDiagnostics.apply()
        }
    }

    var json: String {
        (try? JSONEncoder().encode(self)).flatMap { String(data: $0, encoding: .utf8) } ?? "{}"
    }
}
