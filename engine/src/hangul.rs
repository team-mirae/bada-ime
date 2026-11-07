//! Hangul syllables and jamo: composing, decomposing, double finals.

pub const INITIALS: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];
pub const MEDIALS: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];
/// Index 0 is "no final".
pub const FINALS: [char; 28] = [
    '\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ',
    'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

/// Double finals typed as two consonants (2-set).
const CLUSTERS: [(char, char, char); 11] = [
    ('ㄱ', 'ㅅ', 'ㄳ'),
    ('ㄴ', 'ㅈ', 'ㄵ'),
    ('ㄴ', 'ㅎ', 'ㄶ'),
    ('ㄹ', 'ㄱ', 'ㄺ'),
    ('ㄹ', 'ㅁ', 'ㄻ'),
    ('ㄹ', 'ㅂ', 'ㄼ'),
    ('ㄹ', 'ㅅ', 'ㄽ'),
    ('ㄹ', 'ㅌ', 'ㄾ'),
    ('ㄹ', 'ㅍ', 'ㄿ'),
    ('ㄹ', 'ㅎ', 'ㅀ'),
    ('ㅂ', 'ㅅ', 'ㅄ'),
];

/// Compound vowels typed as two vowels (2-set).
const COMPOUNDS: [(char, char, char); 7] = [
    ('ㅗ', 'ㅏ', 'ㅘ'),
    ('ㅗ', 'ㅐ', 'ㅙ'),
    ('ㅗ', 'ㅣ', 'ㅚ'),
    ('ㅜ', 'ㅓ', 'ㅝ'),
    ('ㅜ', 'ㅔ', 'ㅞ'),
    ('ㅜ', 'ㅣ', 'ㅟ'),
    ('ㅡ', 'ㅣ', 'ㅢ'),
];

pub fn is_vowel(c: char) -> bool {
    MEDIALS.contains(&c)
}

pub fn cluster(a: char, b: char) -> Option<char> {
    CLUSTERS
        .iter()
        .find(|(x, y, _)| *x == a && *y == b)
        .map(|t| t.2)
}

pub fn compound(a: char, b: char) -> Option<char> {
    COMPOUNDS
        .iter()
        .find(|(x, y, _)| *x == a && *y == b)
        .map(|t| t.2)
}

/// A syllable from compatibility jamo; `final_` may be absent. None if a jamo can't take its role.
pub fn compose(initial: char, medial: char, final_: Option<char>) -> Option<char> {
    let l = INITIALS.iter().position(|&c| c == initial)?;
    let v = MEDIALS.iter().position(|&c| c == medial)?;
    let t = match final_ {
        None => 0,
        Some(f) => FINALS.iter().skip(1).position(|&c| c == f)? + 1,
    };
    char::from_u32(0xAC00 + ((l * 21 + v) * 28 + t) as u32)
}

pub fn is_syllable(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
}

/// (initial, medial, final) of a precomposed syllable.
pub fn decompose(c: char) -> Option<(char, char, Option<char>)> {
    if !is_syllable(c) {
        return None;
    }
    let n = c as u32 - 0xAC00;
    let t = (n % 28) as usize;
    Some((
        INITIALS[(n / 588) as usize],
        MEDIALS[((n % 588) / 28) as usize],
        if t == 0 { None } else { Some(FINALS[t]) },
    ))
}

/// The initial of each syllable of `word` (non-syllables as they are): 안녕 → ㅇㄴ.
pub fn choseong(word: &str) -> String {
    word.chars()
        .map(|c| decompose(c).map_or(c, |d| d.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        assert_eq!(compose('ㅎ', 'ㅏ', Some('ㄴ')), Some('한'));
        assert_eq!(decompose('한'), Some(('ㅎ', 'ㅏ', Some('ㄴ'))));
        assert_eq!(compose('ㄱ', 'ㅏ', Some('ㅄ')), Some('값'));
        assert_eq!(compose('ㄸ', 'ㅏ', Some('ㄸ')), None);
        assert_eq!(choseong("안녕하세요"), "ㅇㄴㅎㅅㅇ");
    }
}
