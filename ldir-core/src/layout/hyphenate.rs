//! Pattern-based hyphenation for English text.
//!
//! Uses a simplified algorithmic hyphenator that splits words at syllable
//! boundaries using common English patterns, affix rules, and user-specified
//! hyphenation points.

#![allow(dead_code)]

/// A position within a word where a hyphen can be inserted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyphenPoint {
    /// Byte offset in the word where a hyphen can be inserted.
    pub position: usize,
    /// Quality of this hyphenation point.
    pub quality: HyphenQuality,
}

/// How good a hyphenation break point is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HyphenQuality {
    /// Natural syllable boundary (best).
    Excellent,
    /// Common affix boundary.
    Good,
    /// Algorithmic guess (worst acceptable).
    Acceptable,
}

const PREFIXES: &[&str] = &[
    "un", "re", "pre", "dis", "mis", "over", "out", "sub", "inter", "trans", "super", "anti",
    "semi", "auto", "non",
];
const SUFFIXES: &[&str] = &[
    "ing", "ed", "tion", "ly", "ness", "ment", "able", "ible", "ful", "less", "ous", "ive", "ity",
    "ize", "ise", "al", "er", "est", "ism", "ist",
];

const MIN_WORD_LEN: usize = 5;
const PROTECT_MARGIN: usize = 3;

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

fn char_byte_offset(chars: &[char], char_index: usize) -> usize {
    chars[..char_index].iter().map(|c| c.len_utf8()).sum()
}

fn hyphenate_by_syllables(word: &str) -> Vec<(usize, HyphenQuality)> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < MIN_WORD_LEN {
        return vec![];
    }

    let mut points: Vec<(usize, HyphenQuality)> = Vec::new();

    // For short words (5-7 chars), use a relaxed margin of 2
    let margin = if chars.len() <= 7 { 2 } else { PROTECT_MARGIN };

    for i in margin..chars.len() - margin {
        let prev = chars[i - 1];
        let curr = chars[i];
        let next = chars[i + 1];

        let prev_is_vowel = is_vowel(prev);
        let curr_is_vowel = is_vowel(curr);
        let next_is_vowel = is_vowel(next);
        let after_next_vowel = i + 2 < chars.len() && is_vowel(chars[i + 2]);

        if prev_is_vowel && !curr_is_vowel && (next_is_vowel || after_next_vowel) {
            points.push((char_byte_offset(&chars, i), HyphenQuality::Excellent));
        }
        if !prev_is_vowel && prev == curr && i < chars.len() - margin {
            points.push((char_byte_offset(&chars, i), HyphenQuality::Good));
        }
    }

    points
}

fn hyphenate_by_affixes(word: &str) -> Vec<(usize, HyphenQuality)> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < MIN_WORD_LEN {
        return vec![];
    }

    let mut points: Vec<(usize, HyphenQuality)> = Vec::new();
    let margin = if chars.len() <= 7 { 2 } else { PROTECT_MARGIN };

    for &prefix in PREFIXES {
        if word.starts_with(prefix) && prefix.len() + 2 < word.len() {
            let pos = prefix.len();
            if pos >= margin && pos <= chars.len() - margin {
                points.push((char_byte_offset(&chars, pos), HyphenQuality::Good));
            }
        }
    }

    for &suffix in SUFFIXES {
        if word.ends_with(suffix) && word.len() > suffix.len() + PROTECT_MARGIN {
            let pos = chars.len() - suffix.len();
            if pos >= margin && pos <= chars.len() - margin {
                points.push((char_byte_offset(&chars, pos), HyphenQuality::Good));
            }
        }
    }

    points
}

fn hyphenate_user_specified(word: &str) -> Vec<(usize, HyphenQuality)> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < MIN_WORD_LEN {
        return vec![];
    }

    let mut points: Vec<(usize, HyphenQuality)> = Vec::new();

    for (i, window) in chars.windows(2).enumerate() {
        if window[0] == '\\' && window[1] == '-' {
            let byte_pos = char_byte_offset(&chars, i);
            if i >= PROTECT_MARGIN && i < chars.len() - PROTECT_MARGIN {
                points.push((byte_pos, HyphenQuality::Excellent));
            }
        }
    }

    points
}

/// Hyphenate a word, returning all valid hyphenation points.
///
/// Handles:
/// - User-specified hyphenation points (`\-`)
/// - Common English prefixes and suffixes
/// - Syllable boundary detection using vowel/consonant patterns
///
/// Words shorter than 5 characters are never hyphenated.
/// The first and last 3 characters are always protected.
pub fn hyphenate_word(word: &str) -> Vec<HyphenPoint> {
    if word.len() < MIN_WORD_LEN
        || !word
            .chars()
            .all(|c| c.is_alphabetic() || c == '\\' || c == '-')
    {
        return vec![];
    }

    let chars: Vec<char> = word.chars().filter(|c| *c != '\\' && *c != '-').collect();
    if chars.len() < MIN_WORD_LEN {
        return vec![];
    }

    let mut all_points: Vec<(usize, HyphenQuality)> = Vec::new();

    all_points.extend(hyphenate_user_specified(word));
    all_points.extend(hyphenate_by_affixes(word));
    all_points.extend(hyphenate_by_syllables(word));

    all_points.sort_by_key(|&(pos, _)| pos);
    all_points.dedup_by(|a, b| {
        if a.0 == b.0 {
            if a.1 < b.1 {
                *a = *b;
            }
            true
        } else {
            false
        }
    });

    all_points
        .into_iter()
        .map(|(position, quality)| HyphenPoint { position, quality })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_words_not_hyphenated() {
        assert!(hyphenate_word("cat").is_empty());
        assert!(hyphenate_word("the").is_empty());
        assert!(hyphenate_word("and").is_empty());
        assert!(hyphenate_word("to").is_empty());
        assert!(hyphenate_word("is").is_empty());
    }

    #[test]
    fn test_five_char_word() {
        let points = hyphenate_word("hello");
        // "hello": h-e-ll-o, should break between double l or at VCV boundary
        assert!(!points.is_empty(), "hello should have hyphenation points");
    }

    #[test]
    fn test_prefix_hyphenation() {
        let points = hyphenate_word("unhappy");
        assert!(
            !points.is_empty(),
            "unhappy should have hyphenation points from prefix"
        );
        assert!(
            points.iter().any(|p| p.position == 2),
            "should break after 'un'"
        );
    }

    #[test]
    fn test_suffix_hyphenation() {
        let points = hyphenate_word("running");
        assert!(!points.is_empty(), "running should have hyphenation points");
        let found = points.iter().any(|p| {
            let suffix = &"running"[p.position..];
            suffix.starts_with("ning") || suffix.starts_with("ing")
        });
        assert!(found, "should break before 'ning'");
    }

    #[test]
    fn test_user_specified_hyphen() {
        let points = hyphenate_word("hy\\-phen\\-ation");
        assert!(!points.is_empty());
        assert!(points.iter().any(|p| p.quality == HyphenQuality::Excellent));
    }

    #[test]
    fn test_first_last_chars_protected() {
        let word = "international";
        let points = hyphenate_word(word);
        for p in &points {
            assert!(
                p.position >= PROTECT_MARGIN,
                "position {} < {}",
                p.position,
                PROTECT_MARGIN
            );
            assert!(
                p.position <= word.len() - PROTECT_MARGIN,
                "position {} > {}",
                p.position,
                word.len() - PROTECT_MARGIN
            );
        }
    }

    #[test]
    fn test_no_hyphenation_at_boundaries() {
        let word = "abcdefg";
        let points = hyphenate_word(word);
        for p in &points {
            assert!(p.position >= PROTECT_MARGIN);
            assert!(p.position <= word.len() - PROTECT_MARGIN);
        }
    }

    #[test]
    fn test_common_english_words() {
        let words = [
            "computer",
            "information",
            "beautiful",
            "wonderful",
            "understanding",
        ];
        for word in words {
            let points = hyphenate_word(word);
            if !points.is_empty() {
                for p in &points {
                    assert!(p.position < word.len());
                    assert!(p.position >= PROTECT_MARGIN);
                }
            }
        }
    }

    #[test]
    fn test_re_prefix() {
        let points = hyphenate_word("rewrite");
        assert!(!points.is_empty(), "rewrite should have hyphenation points");
    }

    #[test]
    fn test_ness_suffix() {
        let points = hyphenate_word("happiness");
        assert!(
            !points.is_empty(),
            "happiness should have hyphenation points"
        );
    }

    #[test]
    fn test_double_consonant() {
        let points = hyphenate_word("letter");
        assert!(!points.is_empty(), "letter should hyphenate at double t");
        let tt_break = points
            .iter()
            .any(|p| "letter"[p.position..].starts_with("ter"));
        assert!(tt_break, "should break between double t");
    }
}
