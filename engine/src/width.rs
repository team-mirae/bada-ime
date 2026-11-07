//! Full-width and half-width (半角/全角, spec §6.2–6.3). Ported from JIS Moonshot (width.rs).

/// Full-width (全角) or half-width (半角).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    Full,
    Half,
}

/// Half-width katakana U+FF66–U+FF9D and their full-width forms, in order.
const HALF_KANA: &str = "ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ";
const FULL_KANA: &str = "ヲァィゥェォャュョッーアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワン";
const VOICED_MARK: char = 'ﾞ';
const SEMI_VOICED_MARK: char = 'ﾟ';

/// Katakana with a voiced form one code point up (ガ for カ).
const VOICEABLE: &str = "カキクケコサシスセソタチツテトハヒフヘホ";
/// Katakana with a semi-voiced form two code points up (パ for ハ).
const SEMI_VOICEABLE: &str = "ハヒフヘホ";

fn offset(c: char, by: u32) -> char {
    char::from_u32(c as u32 + by).unwrap_or(c)
}

/// A full-width katakana as half-width: one or two characters (ガ → ｶﾞ). `None` if it has none.
fn half_kana(c: char) -> Option<String> {
    let pick = |base: char| {
        FULL_KANA
            .chars()
            .position(|k| k == base)
            .and_then(|i| HALF_KANA.chars().nth(i))
    };
    if let Some(h) = pick(c) {
        return Some(h.to_string());
    }
    if c == 'ヴ' {
        return Some(format!("ｳ{VOICED_MARK}"));
    }
    let base = VOICEABLE.chars().find(|&b| offset(b, 1) == c);
    if let Some(h) = base.and_then(pick) {
        return Some(format!("{h}{VOICED_MARK}"));
    }
    let base = SEMI_VOICEABLE.chars().find(|&b| offset(b, 2) == c);
    base.and_then(pick)
        .map(|h| format!("{h}{SEMI_VOICED_MARK}"))
}

/// `text` in `width`: every character with a form in that width takes it, the rest stay
/// (spec §5.3's pairs). Half-width also makes kana half-width katakana (かな → ｶﾅ, ガ → ｶﾞ), and
/// full-width puts half-width katakana back (ｶﾞ → ガ); Hangul has no other width.
pub fn convert(text: &str, width: Width) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match width {
            Width::Full => {
                if let Some(i) = HALF_KANA.chars().position(|k| k == c) {
                    let full = FULL_KANA.chars().nth(i).unwrap_or(c);
                    let next = chars.peek().copied();
                    if next == Some(VOICED_MARK) && full == 'ウ' {
                        chars.next();
                        out.push('ヴ');
                    } else if next == Some(VOICED_MARK) && VOICEABLE.contains(full) {
                        chars.next();
                        out.push(offset(full, 1));
                    } else if next == Some(SEMI_VOICED_MARK) && SEMI_VOICEABLE.contains(full) {
                        chars.next();
                        out.push(offset(full, 2));
                    } else {
                        out.push(full);
                    }
                } else if c == VOICED_MARK {
                    out.push('゛');
                } else if c == SEMI_VOICED_MARK {
                    out.push('゜');
                } else {
                    out.push(full_width(c));
                }
            }
            Width::Half => {
                // Hiragana goes as katakana, as Japanese IMEs make it half-width.
                let kata = if ('\u{3041}'..='\u{3096}').contains(&c) {
                    offset(c, 0x60)
                } else {
                    c
                };
                match (kata, half_kana(kata)) {
                    (_, Some(half)) => out.push_str(&half),
                    ('゛', _) => out.push(VOICED_MARK),
                    ('゜', _) => out.push(SEMI_VOICED_MARK),
                    _ => out.push(half_width(c)),
                }
            }
        }
    }
    out
}

const WIDTH_PAIRS: &[(char, char)] = &[
    ('¢', '￠'),
    ('£', '￡'),
    ('¬', '￢'),
    ('¯', '￣'),
    ('¦', '￤'),
    ('¥', '￥'),
    ('₩', '￦'),
    ('｡', '。'),
    ('｢', '「'),
    ('｣', '」'),
    ('､', '、'),
    ('･', '・'),
];

/// The full-width form of a character, or the character itself when it has none.
pub fn full_width(c: char) -> char {
    match c {
        '!'..='~' => char::from_u32(c as u32 + 0xFEE0).unwrap_or(c),
        ' ' => '\u{3000}',
        _ => WIDTH_PAIRS
            .iter()
            .find(|&&(half, _)| half == c)
            .map_or(c, |&(_, full)| full),
    }
}

/// The half-width form of a character, or the character itself when it has none.
pub fn half_width(c: char) -> char {
    match c {
        '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
        '\u{3000}' => ' ',
        _ => WIDTH_PAIRS
            .iter()
            .find(|&&(_, full)| full == c)
            .map_or(c, |&(half, _)| half),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths() {
        assert_eq!(convert("han 12", Width::Full), "ｈａｎ\u{3000}１２");
        assert_eq!(convert("ｈａｎ１２", Width::Half), "han12");
        assert_eq!(convert("ガ", Width::Half), "ｶﾞ");
        assert_eq!(convert("한", Width::Full), "한");
    }
}
