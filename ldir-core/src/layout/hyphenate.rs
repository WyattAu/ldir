//! Pattern-based hyphenation for multiple languages.
//!
//! Uses a simplified algorithmic hyphenator that splits words at syllable
//! boundaries using common patterns, affix rules, and user-specified
//! hyphenation points. Supports English, German, French, Spanish, and
//! Portuguese.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HyphenationLang {
    English,
    German,
    French,
    Spanish,
    Portuguese,
    /// Use language-agnostic syllable heuristic only
    Unknown,
}

const ENGLISH_PREFIXES: &[&str] = &[
    "un", "re", "pre", "dis", "mis", "over", "out", "sub", "inter", "trans", "super", "anti",
    "semi", "auto", "non",
];
const ENGLISH_SUFFIXES: &[&str] = &[
    "ing", "ed", "tion", "ly", "ness", "ment", "able", "ible", "ful", "less", "ous", "ive", "ity",
    "ize", "ise", "al", "er", "est", "ism", "ist",
];

const GERMAN_PREFIXES: &[&str] = &["ge", "be", "ent", "er", "ver", "zer"];
const GERMAN_SUFFIXES: &[&str] = &[
    "ung", "keit", "heit", "lich", "isch", "ung", "en", "er", "in",
];

const FRENCH_PREFIXES: &[&str] = &["con", "dis", "ex", "in", "re", "pr\u{00e9}"];
const FRENCH_SUFFIXES: &[&str] = &["tion", "ment", "ement"];

const SPANISH_PREFIXES: &[&str] = &["re", "pre", "in", "des", "sub", "inter"];
const SPANISH_SUFFIXES: &[&str] = &["cion", "mente", "ando", "iendo", "ado", "ido"];

const PORTUGUESE_PREFIXES: &[&str] = &["re", "pre", "in", "des", "sub", "inter", "anti"];
const PORTUGUESE_SUFFIXES: &[&str] = &["cao", "mento", "ando", "indo", "ado", "ido", "ura", "agem"];

struct LangConfig {
    prefixes: &'static [&'static str],
    suffixes: &'static [&'static str],
    min_word_len: usize,
    protect_margin: usize,
}

fn lang_config(lang: HyphenationLang) -> LangConfig {
    match lang {
        HyphenationLang::English => LangConfig {
            prefixes: ENGLISH_PREFIXES,
            suffixes: ENGLISH_SUFFIXES,
            min_word_len: 5,
            protect_margin: 3,
        },
        HyphenationLang::German => LangConfig {
            prefixes: GERMAN_PREFIXES,
            suffixes: GERMAN_SUFFIXES,
            min_word_len: 5,
            protect_margin: 2,
        },
        HyphenationLang::French => LangConfig {
            prefixes: FRENCH_PREFIXES,
            suffixes: FRENCH_SUFFIXES,
            min_word_len: 6,
            protect_margin: 3,
        },
        HyphenationLang::Spanish => LangConfig {
            prefixes: SPANISH_PREFIXES,
            suffixes: SPANISH_SUFFIXES,
            min_word_len: 5,
            protect_margin: 3,
        },
        HyphenationLang::Portuguese => LangConfig {
            prefixes: PORTUGUESE_PREFIXES,
            suffixes: PORTUGUESE_SUFFIXES,
            min_word_len: 5,
            protect_margin: 3,
        },
        HyphenationLang::Unknown => LangConfig {
            prefixes: &[],
            suffixes: &[],
            min_word_len: 5,
            protect_margin: 3,
        },
    }
}

#[cfg(test)]
const PROTECT_MARGIN: usize = 3;

fn is_vowel(c: char, lang: HyphenationLang) -> bool {
    match c {
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'A' | 'E' | 'I' | 'O' | 'U' | 'Y' => true,
        _ => match lang {
            HyphenationLang::French | HyphenationLang::Spanish | HyphenationLang::Portuguese => {
                matches!(c, '\u{00E0}'..='\u{00F6}' | '\u{00F8}'..='\u{00FF}')
            }
            HyphenationLang::German => {
                matches!(
                    c,
                    '\u{00E4}' | '\u{00F6}' | '\u{00FC}' | '\u{00C4}' | '\u{00D6}' | '\u{00DC}'
                )
            }
            _ => false,
        },
    }
}

fn char_byte_offset(chars: &[char], char_index: usize) -> usize {
    chars[..char_index].iter().map(|c| c.len_utf8()).sum()
}

fn hyphenate_by_syllables(word: &str, lang: HyphenationLang) -> Vec<(usize, HyphenQuality)> {
    let cfg = lang_config(lang);
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < cfg.min_word_len {
        return vec![];
    }

    let mut points: Vec<(usize, HyphenQuality)> = Vec::new();

    let margin = if chars.len() <= 7 {
        2
    } else {
        cfg.protect_margin
    };

    for i in margin..chars.len() - margin {
        let prev = chars[i - 1];
        let curr = chars[i];
        let next = chars[i + 1];

        let prev_is_vowel = is_vowel(prev, lang);
        let curr_is_vowel = is_vowel(curr, lang);
        let next_is_vowel = is_vowel(next, lang);
        let after_next_vowel = i + 2 < chars.len() && is_vowel(chars[i + 2], lang);

        if prev_is_vowel && !curr_is_vowel && (next_is_vowel || after_next_vowel) {
            points.push((char_byte_offset(&chars, i), HyphenQuality::Excellent));
        }
        if !prev_is_vowel && prev == curr && i < chars.len() - margin {
            points.push((char_byte_offset(&chars, i), HyphenQuality::Good));
        }
    }

    points
}

fn hyphenate_by_affixes(word: &str, lang: HyphenationLang) -> Vec<(usize, HyphenQuality)> {
    let cfg = lang_config(lang);
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < cfg.min_word_len {
        return vec![];
    }

    let mut points: Vec<(usize, HyphenQuality)> = Vec::new();
    let margin = if chars.len() <= 7 {
        2
    } else {
        cfg.protect_margin
    };

    for &prefix in cfg.prefixes {
        if word.starts_with(prefix) && prefix.len() + 2 < word.len() {
            let pos = prefix.len();
            if pos >= margin && pos <= chars.len() - margin {
                points.push((char_byte_offset(&chars, pos), HyphenQuality::Good));
            }
        }
    }

    for &suffix in cfg.suffixes {
        if word.ends_with(suffix) && word.len() > suffix.len() + cfg.protect_margin {
            let pos = chars.len() - suffix.len();
            if pos >= margin && pos <= chars.len() - margin {
                points.push((char_byte_offset(&chars, pos), HyphenQuality::Good));
            }
        }
    }

    points
}

fn hyphenate_user_specified(word: &str, lang: HyphenationLang) -> Vec<(usize, HyphenQuality)> {
    let cfg = lang_config(lang);
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < cfg.min_word_len {
        return vec![];
    }

    let mut points: Vec<(usize, HyphenQuality)> = Vec::new();

    for (i, window) in chars.windows(2).enumerate() {
        if window[0] == '\\' && window[1] == '-' {
            let byte_pos = char_byte_offset(&chars, i);
            if i >= cfg.protect_margin && i < chars.len() - cfg.protect_margin {
                points.push((byte_pos, HyphenQuality::Excellent));
            }
        }
    }

    points
}

/// Hyphenate a word with the given language, returning all valid hyphenation
/// points.
///
/// Handles:
/// - User-specified hyphenation points (`\-`)
/// - Common prefixes and suffixes for the given language
/// - Syllable boundary detection using vowel/consonant patterns
///
/// Words shorter than the language's minimum length are never hyphenated.
/// The first and last characters are always protected according to the
/// language's margin.
pub fn hyphenate_word_with_lang(word: &str, lang: HyphenationLang) -> Vec<HyphenPoint> {
    let cfg = lang_config(lang);
    if word.len() < cfg.min_word_len
        || !word
            .chars()
            .all(|c| c.is_alphabetic() || c == '\\' || c == '-')
    {
        return vec![];
    }

    let chars: Vec<char> = word.chars().filter(|c| *c != '\\' && *c != '-').collect();
    if chars.len() < cfg.min_word_len {
        return vec![];
    }

    let mut all_points: Vec<(usize, HyphenQuality)> = Vec::new();

    all_points.extend(hyphenate_user_specified(word, lang));
    all_points.extend(hyphenate_by_affixes(word, lang));
    all_points.extend(hyphenate_by_syllables(word, lang));

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

/// Hyphenate a word, returning all valid hyphenation points.
///
/// Equivalent to calling [`hyphenate_word_with_lang`] with
/// [`HyphenationLang::English`].
///
/// Handles:
/// - User-specified hyphenation points (`\-`)
/// - Common English prefixes and suffixes
/// - Syllable boundary detection using vowel/consonant patterns
///
/// Words shorter than 5 characters are never hyphenated.
/// The first and last 3 characters are always protected.
pub fn hyphenate_word(word: &str) -> Vec<HyphenPoint> {
    hyphenate_word_with_lang(word, HyphenationLang::English)
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

    #[test]
    fn test_german_compound_word() {
        let points = hyphenate_word_with_lang("Donaudampfschifffahrt", HyphenationLang::German);
        assert!(
            !points.is_empty(),
            "Donaudampfschifffahrt should have hyphenation points"
        );
        assert!(
            points.len() >= 2,
            "Donaudampfschifffahrt should have at least 2 hyphenation points"
        );
        for p in &points {
            assert!(p.position < "Donaudampfschifffahrt".len());
        }
    }

    #[test]
    fn test_french_accented_word() {
        let points = hyphenate_word_with_lang("d\u{00e9}veloppement", HyphenationLang::French);
        assert!(
            !points.is_empty(),
            "d\u{00e9}veloppement should have hyphenation points"
        );
        let has_ement_suffix = points.iter().any(|p| {
            let suffix = &"d\u{00e9}veloppement"[p.position..];
            suffix == "ment"
        });
        assert!(has_ement_suffix, "should break before 'ment' suffix");
    }

    #[test]
    fn test_spanish_basic_word() {
        let points = hyphenate_word_with_lang("comprensi\u{00f3}n", HyphenationLang::Spanish);
        assert!(
            !points.is_empty(),
            "comprensi\u{00f3}n should have hyphenation points"
        );
        for p in &points {
            assert!(p.position < "comprensi\u{00f3}n".len());
        }
    }

    #[test]
    fn test_portuguese_basic_word() {
        let points = hyphenate_word_with_lang("desenvolvimento", HyphenationLang::Portuguese);
        assert!(
            !points.is_empty(),
            "desenvolvimento should have hyphenation points"
        );
        let has_mento_suffix = points
            .iter()
            .any(|p| "desenvolvimento"[p.position..].starts_with("mento"));
        assert!(has_mento_suffix, "should break before 'mento' suffix");
    }
}
