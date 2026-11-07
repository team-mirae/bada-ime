//! The hanja dictionary (from JIS Korean, `engine/dict/hanja.tsv`: libhangul and Unihan readings).

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub hanja: String,
    /// Meaning and reading for single characters (學: 배울 학). Often empty.
    pub comment: String,
}

/// Packed: readings sorted by bytes in one string, each with a range of candidates, whose hanja and
/// comments sit in a second string as `hanja\tcomment` (a few MB against ~50 in hash maps).
#[derive(Debug, Default)]
pub struct Dictionary {
    readings: String,
    /// Reading `i` is `readings[starts[i]..starts[i + 1]]`.
    starts: Vec<u32>,
    /// Reading `i`'s candidates are `first[i]..first[i + 1]`.
    first: Vec<u32>,
    candidates: String,
    /// Candidate `j` is `candidates[cstarts[j]..cstarts[j + 1]]`.
    cstarts: Vec<u32>,
    /// Longest reading, in syllables.
    longest: usize,
}

impl Dictionary {
    /// Parses `reading<TAB>hanja<TAB>comment` lines. Malformed lines are skipped; a reading's
    /// candidates keep file order, duplicates dropped.
    pub fn parse(tsv: &str) -> Dictionary {
        let mut rows: Vec<(&str, &str, &str)> = tsv
            .lines()
            .filter_map(|line| {
                let mut f = line.split('\t');
                let (reading, hanja) = (f.next()?, f.next()?);
                (!reading.is_empty() && !hanja.is_empty()).then(|| (reading, hanja, f.next().unwrap_or("")))
            })
            .collect();
        rows.sort_by(|a, b| a.0.cmp(b.0)); // stable: candidate order kept
        let mut d = Dictionary::default();
        let mut i = 0;
        while i < rows.len() {
            let reading = rows[i].0;
            d.starts.push(d.readings.len() as u32);
            d.readings.push_str(reading);
            d.first.push(d.cstarts.len() as u32);
            d.longest = d.longest.max(reading.chars().count());
            let mut seen: Vec<&str> = Vec::new();
            while i < rows.len() && rows[i].0 == reading {
                let (_, hanja, comment) = rows[i];
                if !seen.contains(&hanja) {
                    seen.push(hanja);
                    d.cstarts.push(d.candidates.len() as u32);
                    d.candidates.push_str(hanja);
                    d.candidates.push('\t');
                    d.candidates.push_str(comment);
                }
                i += 1;
            }
        }
        d.starts.push(d.readings.len() as u32);
        d.first.push(d.cstarts.len() as u32);
        d.cstarts.push(d.candidates.len() as u32);
        d.readings.shrink_to_fit();
        d.candidates.shrink_to_fit();
        d
    }

    pub fn len(&self) -> usize {
        self.first.len().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn reading(&self, i: usize) -> &str {
        &self.readings[self.starts[i] as usize..self.starts[i + 1] as usize]
    }

    /// The candidates of `reading`, in file order; empty if it has none.
    pub fn lookup(&self, reading: &str) -> Vec<Candidate> {
        let (mut lo, mut hi) = (0, self.len());
        while lo < hi {
            let mid = (lo + hi) / 2;
            match self.reading(mid).cmp(reading) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => {
                    return (self.first[mid] as usize..self.first[mid + 1] as usize)
                        .map(|j| {
                            let text = &self.candidates[self.cstarts[j] as usize..self.cstarts[j + 1] as usize];
                            let (hanja, comment) = text.split_once('\t').unwrap_or((text, ""));
                            Candidate { hanja: hanja.to_string(), comment: comment.to_string() }
                        })
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// The longest trailing run of Hangul syllables in `text` that has candidates:
    /// "나는학교" → "학교". Returns the reading and its candidates.
    pub fn longest_suffix<'a>(&self, text: &'a str) -> Option<(&'a str, Vec<Candidate>)> {
        let syllables: Vec<usize> = text
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_syllable(*c))
            .map(|(i, _)| i)
            .collect();
        syllables.iter().take(self.longest).rev().find_map(|&start| {
            let reading = &text[start..];
            let candidates = self.lookup(reading);
            (!candidates.is_empty()).then_some((reading, candidates))
        })
    }
}

fn is_syllable(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "가\t可\t옳을 가\n가\t家\t집 가\n학교\t學校\t\n학\t學\t배울 학\n교\t校\t학교 교\n\
                          broken line\n가\t可\tduplicate\n";

    #[test]
    fn parses_and_keeps_order() {
        let d = Dictionary::parse(SAMPLE);
        assert_eq!(d.len(), 4);
        let found = d.lookup("가");
        let hanja: Vec<_> = found.iter().map(|c| c.hanja.as_str()).collect();
        assert_eq!(hanja, ["可", "家"]);
        assert_eq!(d.lookup("가")[0].comment, "옳을 가");
        assert!(d.lookup("없음").is_empty());
    }

    #[test]
    fn longest_suffix_prefers_words() {
        let d = Dictionary::parse(SAMPLE);
        assert_eq!(d.longest_suffix("나는학교").map(|(r, _)| r), Some("학교"));
        assert_eq!(d.longest_suffix("나는교").map(|(r, _)| r), Some("교"));
        assert_eq!(d.longest_suffix("학교!"), None);
        assert_eq!(d.longest_suffix("ㄱ"), None);
        assert_eq!(d.longest_suffix(""), None);
    }
}
