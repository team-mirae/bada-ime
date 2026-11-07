//! Physical keys, named by their W3C `KeyboardEvent.code` (spec §2 of JIS Korean).

macro_rules! keys {
    ($($name:ident = $code:literal),* $(,)?) => {
        /// A physical key position on a JIS keyboard.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Key { $($name),* }

        impl Key {
            pub const ALL: &'static [Key] = &[$(Key::$name),*];

            /// The canonical `KeyboardEvent.code` name.
            pub fn code(self) -> &'static str {
                match self { $(Key::$name => $code),* }
            }

            /// Any `KeyboardEvent.code`; unknown codes are [`Key::Other`].
            pub fn from_code(code: &str) -> Key {
                match code { $($code => Key::$name,)* _ => Key::Other }
            }
        }
    };
}

keys! {
    Digit1 = "Digit1", Digit2 = "Digit2", Digit3 = "Digit3", Digit4 = "Digit4", Digit5 = "Digit5",
    Digit6 = "Digit6", Digit7 = "Digit7", Digit8 = "Digit8", Digit9 = "Digit9", Digit0 = "Digit0",
    Minus = "Minus", Equal = "Equal", IntlYen = "IntlYen",
    KeyQ = "KeyQ", KeyW = "KeyW", KeyE = "KeyE", KeyR = "KeyR", KeyT = "KeyT",
    KeyY = "KeyY", KeyU = "KeyU", KeyI = "KeyI", KeyO = "KeyO", KeyP = "KeyP",
    BracketLeft = "BracketLeft", BracketRight = "BracketRight",
    KeyA = "KeyA", KeyS = "KeyS", KeyD = "KeyD", KeyF = "KeyF", KeyG = "KeyG",
    KeyH = "KeyH", KeyJ = "KeyJ", KeyK = "KeyK", KeyL = "KeyL",
    Semicolon = "Semicolon", Quote = "Quote", Backslash = "Backslash",
    KeyZ = "KeyZ", KeyX = "KeyX", KeyC = "KeyC", KeyV = "KeyV",
    KeyB = "KeyB", KeyN = "KeyN", KeyM = "KeyM",
    Comma = "Comma", Period = "Period", Slash = "Slash", IntlRo = "IntlRo",
    // 半角/全角 (left of 1 on a full JIS board)
    Backquote = "Backquote",
    F6 = "F6", F7 = "F7", F8 = "F8", F9 = "F9", F10 = "F10",
    Space = "Space", Enter = "Enter", Backspace = "Backspace", Tab = "Tab", Escape = "Escape",
    ArrowUp = "ArrowUp", ArrowDown = "ArrowDown", ArrowLeft = "ArrowLeft", ArrowRight = "ArrowRight",
    // かな and 英数
    Lang1 = "Lang1", Lang2 = "Lang2",
    Other = "Other",
}

impl Key {
    /// The 48 keys of the JIS kana space, the only keys layouts type on.
    pub fn is_kana_space(self) -> bool {
        (self as usize) <= (Key::IntlRo as usize)
    }

    pub fn is_letter(self) -> bool {
        self.code().len() == 4 && self.code().starts_with("Key")
    }

    /// F6–F10, the F keys bada ime intercepts (spec §6.5).
    pub fn f_index(self) -> Option<u8> {
        match self {
            Key::F6 => Some(6),
            Key::F7 => Some(7),
            Key::F8 => Some(8),
            Key::F9 => Some(9),
            Key::F10 => Some(10),
            _ => None,
        }
    }
}

/// Modifier state for one key event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(default)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Mods {
    pub const NONE: Mods = Mods {
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
    };
    pub const SHIFT: Mods = Mods {
        shift: true,
        ctrl: false,
        alt: false,
        meta: false,
    };

    /// Ctrl or Cmd: a shortcut for the app.
    pub fn is_shortcut(self) -> bool {
        self.ctrl || self.meta
    }
}
