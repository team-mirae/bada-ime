import Foundation

/// A henkan profile (spec §6.6): its own candidates, or a link to another profile.
struct BadaProfile: Codable, Equatable {
    var link: String?
    var enabled: [UInt32] = []
    /// This profile's own F keys; nil: the same as かな's (`intercept_f`, `fkeys`).
    var f_keys: BadaFKeys?
}

/// F6–F10 for one profile (spec §6.5): intercepted or not, and each key's binding.
struct BadaFKeys: Codable, Equatable {
    var intercept = true
    var keys: [String: String] = [:]
}

/// The user's settings (spec §5.3, §6.4, §6.5), stored as the engine's JSON in UserDefaults.
struct BadaSettings: Codable, Equatable {
    /// Henkan candidates (layout ids), in order.
    var enabled: [UInt32] = [13]
    var default_muhenkan: UInt32 = 0
    /// Henkan profiles (spec §6.6): off by default; 変換 and カタカナ need the key tap (Accessibility).
    var use_profiles = false
    /// "mac-kana", "henkan", "katakana" → own candidates or a link. Missing: henkan and katakana link
    /// to mac-kana, whose own list is `enabled`.
    var profiles: [String: BadaProfile] = [:]
    /// Layout id → weight (spec §5.3): a multiplier on every entry that layout raises. Missing: 1.
    var weights: [String: Double] = [:]
    /// "henkan", "katakana" → the macOS key code that selects it (recorded in Settings).
    var profile_keys: [String: Int] = [:]
    /// What かな does in henkan mode in each profile but mac-kana (spec §6.6): "henkan", "katakana"
    /// → one of `KanaInProfile`'s raw values. Missing: next-or-mac-kana.
    var kana_in_profile: [String: String] = [:]
    /// The order かな cycles profiles in, with `kana_in_profile` = "cycle".
    var profile_order = ["mac-kana", "henkan", "katakana"]
    /// Emoji skin tones in the order layout 150 offers them; a tone left out isn't offered.
    var skin_tones = BadaSettings.allTones
    static let allTones = ["none", "light", "medium-light", "medium", "medium-dark", "dark"]
    /// Negative menu entries (−1 hiragana, −2 katakana for Japanese): off by default.
    var script_forms = false
    /// The one method allowed to add negative menu entries; nil: the lowest enabled that offers them.
    var negative_method: UInt32?
    var intercept_f = true
    /// "F6"…"F10" → "swallow" | "pass" | "<method>.<action>". Nothing is bound by default.
    var fkeys: [String: String] = [:]
    /// Layout 15 (bada roman): Japanese romaji readings as shortcuts (mei 명). On by default.
    var ja_shortcuts = true

    enum CodingKeys: String, CodingKey {
        case enabled, default_muhenkan, weights, skin_tones, use_profiles, profiles, profile_keys, kana_in_profile, profile_order, script_forms, negative_method, intercept_f, fkeys, ja_shortcuts
    }

    init() {}

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        enabled = try c.decodeIfPresent([UInt32].self, forKey: .enabled) ?? [13]
        default_muhenkan = try c.decodeIfPresent(UInt32.self, forKey: .default_muhenkan) ?? 0
        weights = try c.decodeIfPresent([String: Double].self, forKey: .weights) ?? [:]
        skin_tones = try c.decodeIfPresent([String].self, forKey: .skin_tones) ?? BadaSettings.allTones
        use_profiles = try c.decodeIfPresent(Bool.self, forKey: .use_profiles) ?? false
        profiles = try c.decodeIfPresent([String: BadaProfile].self, forKey: .profiles) ?? [:]
        profile_keys = try c.decodeIfPresent([String: Int].self, forKey: .profile_keys) ?? [:]
        if let perProfile = try? c.decodeIfPresent([String: String].self, forKey: .kana_in_profile) {
            kana_in_profile = perProfile
        } else if let one = try? c.decodeIfPresent(String.self, forKey: .kana_in_profile) {
            kana_in_profile = ["henkan": one, "katakana": one]   // one setting for both (an earlier build)
        }
        profile_order = try c.decodeIfPresent([String].self, forKey: .profile_order) ?? ["mac-kana", "henkan", "katakana"]
        script_forms = try c.decodeIfPresent(Bool.self, forKey: .script_forms) ?? false
        negative_method = try c.decodeIfPresent(UInt32.self, forKey: .negative_method)
        intercept_f = try c.decodeIfPresent(Bool.self, forKey: .intercept_f) ?? true
        fkeys = try c.decodeIfPresent([String: String].self, forKey: .fkeys) ?? [:]
        ja_shortcuts = try c.decodeIfPresent(Bool.self, forKey: .ja_shortcuts) ?? true
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
            LanguageKeyTap.refreshProfileKeys(newValue)
            HIDKeys.check(newValue)
            if !newValue.use_profiles { Modes.profile = BadaProfiles.macKana }
        }
    }

    var json: String {
        (try? JSONEncoder().encode(self)).flatMap { String(data: $0, encoding: .utf8) } ?? "{}"
    }

    /// The settings as the engine takes them: with profiles off, every profile is かな's.
    var engineJSON: String {
        var copy = self
        if !use_profiles { copy.profiles = [:] }
        return copy.json
    }
}

/// The henkan profiles' names (spec §6.6) and the keys that select them.
enum BadaProfiles {
    static let macKana = "mac-kana"
    static let all: [(name: String, key: String)] = [("mac-kana", "かな"), ("henkan", "変換"), ("katakana", "カタカナ")]
}

/// What かな does in henkan mode while another profile than mac-kana is in use (spec §6.6).
enum KanaInProfile: String, CaseIterable {
    /// The next menu entry (⇧: previous) when there's a menu to move in; else back to mac-kana.
    case nextOrBack = "next-or-mac-kana"
    case macKana = "mac-kana"
    /// The next profile in `profile_order` (⇧: the previous one).
    case cycle = "cycle"
    case henkan = "henkan"
    case katakana = "katakana"
    /// Leave henkan mode, as 英数.
    case muhenkan = "muhenkan"

    /// The behaviour set for profile `name`.
    static func of(_ settings: BadaSettings, _ name: String) -> KanaInProfile {
        settings.kana_in_profile[name].flatMap(KanaInProfile.init(rawValue:)) ?? .nextOrBack
    }

    var title: String {
        switch self {
        case .nextOrBack: return "Next entry (⇧: previous) if there's a menu, else back to かな's profile"
        case .macKana: return "Back to かな's profile"
        case .cycle: return "Cycle the profiles (⇧: backwards)"
        case .henkan: return "To 変換's profile"
        case .katakana: return "To カタカナ's profile"
        case .muhenkan: return "Leave henkan mode (as 英数)"
        }
    }
}

extension BadaSettings {
    enum FileError: LocalizedError {
        case notSettings
        var errorDescription: String? { "It isn't a bada ime settings file (format bada-settings/1)." }
    }

    /// The settings as a file: `{"format": "bada-settings/1", "settings": {…}}`, readable JSON.
    func exported() throws -> Data {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let settings = try JSONSerialization.jsonObject(with: encoder.encode(self))
        let file: [String: Any] = ["format": "bada-settings/1", "settings": settings]
        return try JSONSerialization.data(withJSONObject: file, options: [.prettyPrinted, .sortedKeys])
    }

    /// Reads a settings file (fields it lacks take their defaults).
    static func imported(_ data: Data) throws -> BadaSettings {
        guard let file = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              file["format"] as? String == "bada-settings/1", let settings = file["settings"] else { throw FileError.notSettings }
        return try JSONDecoder().decode(BadaSettings.self, from: JSONSerialization.data(withJSONObject: settings))
    }
}

