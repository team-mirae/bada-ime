// Checks ProfileKeyLogic (spec §6.6). Run: tools/test_profile_keys.sh
var failures = 0
func check(_ got: ProfileKeyAction, _ want: ProfileKeyAction, _ what: String) {
    if got != want { failures += 1; print("FAIL \(what): \(got), want \(want)") } else { print("ok   \(what)") }
}
let order = ["mac-kana", "henkan", "katakana"]
func act(_ key: String, _ cur: String, shift: Bool = false, menu: Bool = false, on: Bool = true, kana: [String: String] = [:], order o: [String] = order) -> ProfileKeyAction {
    ProfileKeyLogic.action(key: key, current: cur, shift: shift, menu: menu, profilesOn: on, kanaIn: kana, order: o)
}
// profiles off: every key steps the menu
check(act("henkan", "mac-kana", on: false), .step(back: false), "profiles off: 変換 steps")
// the same profile's key steps; ⇧ steps back
check(act("mac-kana", "mac-kana"), .step(back: false), "かな in かな's profile: next")
check(act("mac-kana", "mac-kana", shift: true), .step(back: true), "⇧かな in かな's profile: previous")
check(act("henkan", "henkan"), .step(back: false), "変換 in 変換's profile: next")
check(act("katakana", "katakana", shift: true), .step(back: true), "⇧カタカナ in カタカナ's: previous")
// another profile's key switches
check(act("henkan", "mac-kana"), .profile("henkan"), "変換 from かな's: to 変換's")
check(act("katakana", "henkan"), .profile("katakana"), "カタカナ from 変換's: to カタカナ's")
// かな in another profile, per setting
check(act("mac-kana", "henkan", menu: true), .step(back: false), "default, menu: next")
check(act("mac-kana", "henkan", shift: true, menu: true), .step(back: true), "default, menu, ⇧: previous")
check(act("mac-kana", "henkan", menu: false), .profile("mac-kana"), "default, no menu: back to かな's")
check(act("mac-kana", "henkan", menu: true, kana: ["henkan": "mac-kana"]), .profile("mac-kana"), "mac-kana: always back")
check(act("mac-kana", "henkan", kana: ["henkan": "cycle"]), .profile("katakana"), "cycle from 変換: カタカナ")
check(act("mac-kana", "katakana", kana: ["katakana": "cycle"]), .profile("mac-kana"), "cycle from カタカナ: wraps to かな")
check(act("mac-kana", "henkan", shift: true, kana: ["henkan": "cycle"]), .profile("mac-kana"), "⇧ cycle from 変換: かな")
check(act("mac-kana", "henkan", kana: ["henkan": "cycle"], order: ["henkan", "mac-kana", "katakana"]), .profile("mac-kana"), "custom order")
check(act("mac-kana", "katakana", kana: ["katakana": "henkan"]), .profile("henkan"), "to 変換's")
check(act("mac-kana", "henkan", kana: ["henkan": "henkan"]), .step(back: false), "to 変換's from 変換's: next")
check(act("mac-kana", "katakana", kana: ["katakana": "muhenkan"]), .muhenkan, "muhenkan: leave henkan")
check(act("mac-kana", "henkan", kana: ["katakana": "muhenkan"], ), .profile("mac-kana"), "per profile: 変換's on the default")
print(failures == 0 ? "all passed" : "\(failures) failed")
