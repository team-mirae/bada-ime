import Cocoa
import InputMethodKit

// Input methods are background apps: IMKServer accepts client connections, and the system creates
// one BadaInputController per text input client.
let connectionName = Bundle.main.infoDictionary?["InputMethodConnectionName"] as? String ?? "me.sjun.inputmethod.Bada_Connection"
DiagnosticLog.note("launched from \(Bundle.main.bundlePath)")
_ = sharedData
UserDictionary.refresh()
LanguageKeyTap.check()
let server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)
_ = server
NSApplication.shared.run()
