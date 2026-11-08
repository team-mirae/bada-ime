/// What a language key does in henkan mode (spec §6.6), as a pure decision: no input method state,
/// so it can be checked on its own (tools/test_profile_keys.swift).
enum ProfileKeyAction: Equatable {
    /// The next menu entry (`back`: the previous).
    case step(back: Bool)
    /// Switch to this profile (commits what's staged).
    case profile(String)
    /// Leave henkan mode, as 英数.
    case muhenkan
}

enum ProfileKeyLogic {
    static let macKana = "mac-kana"
    static let names = ["mac-kana", "henkan", "katakana"]

    /// `key`: the profile of the key pressed (かな: mac-kana); `current`: the profile in use;
    /// `menu`: whether a menu with more than one row shows; `kanaIn`: what かな does in each other
    /// profile (KanaInProfile raw values; missing: next-or-mac-kana); `order`: the cycle order.
    static func action(key: String, current: String, shift: Bool, menu: Bool, profilesOn: Bool,
                       kanaIn: [String: String], order: [String]) -> ProfileKeyAction {
        guard profilesOn else { return .step(back: shift) }
        var target = key
        if key == macKana && current != macKana {
            switch kanaIn[current] ?? "next-or-mac-kana" {
            case "mac-kana":
                target = macKana
            case "cycle":
                let order = order.filter { names.contains($0) }
                guard !order.isEmpty else { target = macKana; break }
                let at = order.firstIndex(of: current) ?? 0
                target = order[(at + (shift ? order.count - 1 : 1)) % order.count]
            case "henkan":
                target = "henkan"
            case "katakana":
                target = "katakana"
            case "muhenkan":
                return .muhenkan
            default:   // next-or-mac-kana
                if menu { return .step(back: shift) }
                target = macKana
            }
        }
        return target == current ? .step(back: shift) : .profile(target)
    }
}
