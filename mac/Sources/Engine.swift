import Foundation

/// What the engine asks for after an event (engine/src/engine.rs `Output`).
struct EngineOutput: Decodable {
    var commit = ""
    var replace_selection = false
    var preedit = ""
    var menu: [String] = []
    var selected = 0
    var zero = 0
    var consumed = false
    var horizontal = false
    var notes: [String] = []
    var japanese: [Bool] = []
    var need_selection: String?
}

/// The data every engine shares: word lists, hanja, OECD names. Loaded once from the bundle.
let sharedData: OpaquePointer? = {
    func file(_ name: String) -> Data {
        Bundle.main.url(forResource: name, withExtension: nil, subdirectory: "data").flatMap { try? Data(contentsOf: $0) } ?? Data()
    }
    let ko = file("ko-words.tsv"), en = file("en-words.txt"), hanja = file("hanja.tsv"), oecd = file("oecd.tsv")
    let ja = file("ja-words.txt"), kanji = file("kanji.bin"), lm = file("ko-jamo-4gram.bin")
    let symbols = file("symbols.tsv"), emoji = file("emoji.tsv"), unicode = file("unicode-names.tsv")
    let enKo = file("en-ko.tsv")
    return ko.withUnsafeBytes { k in en.withUnsafeBytes { e in hanja.withUnsafeBytes { h in oecd.withUnsafeBytes { o in ja.withUnsafeBytes { j in kanji.withUnsafeBytes { kj in lm.withUnsafeBytes { m in
           symbols.withUnsafeBytes { sy in emoji.withUnsafeBytes { em in unicode.withUnsafeBytes { un in enKo.withUnsafeBytes { ek in
        bada_data_new(k.bindMemory(to: UInt8.self).baseAddress, k.count, e.bindMemory(to: UInt8.self).baseAddress, e.count,
                      h.bindMemory(to: UInt8.self).baseAddress, h.count, o.bindMemory(to: UInt8.self).baseAddress, o.count,
                      j.bindMemory(to: UInt8.self).baseAddress, j.count, kj.bindMemory(to: UInt8.self).baseAddress, kj.count,
                      m.bindMemory(to: UInt8.self).baseAddress, m.count,
                      sy.bindMemory(to: UInt8.self).baseAddress, sy.count, em.bindMemory(to: UInt8.self).baseAddress, em.count,
                      un.bindMemory(to: UInt8.self).baseAddress, un.count,
                      ek.bindMemory(to: UInt8.self).baseAddress, ek.count)
    } } } } } } } } } } }
}()

/// One engine (one per text input client). All behaviour is in the engine.
final class Engine {
    private let handle: OpaquePointer?

    init() {
        handle = bada_new(sharedData)
    }

    deinit {
        bada_free(handle)
    }

    private func decode(_ pointer: UnsafePointer<CChar>?) -> EngineOutput {
        guard let pointer, let data = String(cString: pointer).data(using: .utf8) else { return EngineOutput() }
        return (try? JSONDecoder().decode(EngineOutput.self, from: data)) ?? EngineOutput()
    }

    func key(_ code: String, shift: Bool, ctrl: Bool, alt: Bool, meta: Bool) -> EngineOutput {
        let event: [String: Any] = ["key": code, "shift": shift, "ctrl": ctrl, "alt": alt, "meta": meta]
        let json = (try? JSONSerialization.data(withJSONObject: event)).flatMap { String(data: $0, encoding: .utf8) } ?? "{}"
        return decode(bada_key(handle, json))
    }

    func setMode(_ mode: BadaMode) -> EngineOutput {
        decode(bada_set_mode(handle, mode == .muhenkan ? 0 : 1))
    }

    func setProfile(_ name: String) -> EngineOutput {
        decode(bada_set_profile(handle, name))
    }

    func flush() -> EngineOutput {
        decode(bada_flush(handle))
    }

    func hostSelection(_ text: String) -> EngineOutput {
        decode(bada_host_selection(handle, text))
    }

    func apply(_ settings: String) {
        _ = bada_set_settings(handle, settings)
    }

    /// bada-compatible v1.1 `P_state` (spec §9): the engine's state as JSON, read-only.
    func state() -> String? {
        bada_state(handle).map { String(cString: $0) }
    }

    /// The layouts and every method's F-key actions, as JSON.
    func catalog() -> [String: Any] {
        guard let pointer = bada_catalog(handle), let data = String(cString: pointer).data(using: .utf8) else { return [:] }
        return (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] ?? [:]
    }
}
