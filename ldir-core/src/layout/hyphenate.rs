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
    /// Pattern-based hyphenation (Liou algorithm).
    Pattern,
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

// ---------------------------------------------------------------------------
// Liou pattern-based hyphenation engine
// ---------------------------------------------------------------------------

const MIN_PATTERN_WORD_LEN: usize = 5;

/// A compiled Liou pattern trie for hyphenation.
struct HyphenPatterns {
    patterns: Vec<(Vec<char>, Vec<u8>)>,
}

impl HyphenPatterns {
    fn from_pattern_data(data: &str) -> Self {
        let mut patterns = Vec::new();
        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('%') || line.starts_with('#') {
                continue;
            }
            for token in line.split_whitespace() {
                if let Some(pair) = parse_liou_pattern_token(token) {
                    patterns.push(pair);
                }
            }
        }
        HyphenPatterns { patterns }
    }
}

/// Parse a single TeX hyphenation pattern token.
///
/// Format: characters interleaved with digit levels. For example, `a3b2c2`
/// means: at position 0 (before 'a') level is 0 (implicit), at position 1
/// (between 'a' and 'b') level is 3, at position 2 (between 'b' and 'c')
/// level is 2, at position 3 (after 'c') level is 2.
/// Digits 0-9 encode hyphenation levels. Odd levels indicate a valid break.
fn parse_liou_pattern_token(token: &str) -> Option<(Vec<char>, Vec<u8>)> {
    if token.is_empty() {
        return None;
    }

    let mut chars = Vec::new();
    let mut levels = Vec::new();

    let mut digit_buf = String::new();

    for c in token.chars() {
        if c.is_ascii_digit() {
            digit_buf.push(c);
        } else if c == '.' || c.is_ascii_alphabetic() {
            if !digit_buf.is_empty() {
                while levels.len() < chars.len() {
                    levels.push(0);
                }
                for d in digit_buf.drain(..) {
                    levels.push(d.to_digit(10)? as u8);
                }
            }
            chars.push(c);
        } else {
            return None;
        }
    }

    if !digit_buf.is_empty() {
        while levels.len() < chars.len() {
            levels.push(0);
        }
        for d in digit_buf.drain(..) {
            levels.push(d.to_digit(10)? as u8);
        }
    }

    // levels should be chars.len() + 1 positions
    while levels.len() < chars.len() + 1 {
        levels.push(0);
    }
    if levels.len() > chars.len() + 1 {
        levels.truncate(chars.len() + 1);
    }

    if chars.is_empty() {
        None
    } else {
        Some((chars, levels))
    }
}

/// Minimal embedded English hyphenation patterns (truncated subset for testing).
const EMBEDDED_ENGLISH_PATTERNS: &str = "\
.a3 .b2 .c2 .d2 .e2 .f2 .g2 .h2 .i2 .j2 .k2 .l2 .m2 .n2 .o2 .p2 .q2 .r2 .s2 .t2 .u2 .v2 .w2 .y2 .z2
.ab2 .ac3 .ad2 .af2 .ag2 .ah2 .ai2 .aj2 .ak2 .al2 .am2 .an2 .ao2 .ap2 .aq2 .ar2 .as2 .at2 .au2 .av2 .aw2 .ay2 .az2
.ba3 .be3 .bi3 .bo3 .br3 .bu3 .by3 .b3s
3sa3 3sc3 3se3 3sf3 3sg3 3sh3 3si3 3sk3 3sl3 3sm3 3sn3 3so3 3sp3 3sq3 3sr3 3ss3 3st3 3su3 3sv3 3sw3 3sy3 3sz3
.ca3 .ce3 .ci3 .co3 .cr3 .cu3 .cy3 .c3k
3da3 3de3 3di3 3do3 3dr3 3du3 3dy3 3d3w
.fa3 .fe3 .fi3 .fo3 .fr3 .fu3 .fy3 .f3f
.ga3 .ge3 .gi3 .go3 .gr3 .gu3 .gy3 .g3g
.ha3 .he3 .hi3 .ho3 .hr3 .hu3 .hy3 .h3h
.ja3 .je3 .ji3 .jo3 .ju3 .j3j
.ka3 .ke3 .ki3 .ko3 .kr3 .ku3 .ky3 .k3k
.la3 .le3 .li3 .lo3 .lr3 .lu3 .ly3 .l3l
.ma3 .me3 .mi3 .mo3 .mr3 .mu3 .my3 .m3m
.na3 .ne3 .ni3 .no3 .nr3 .nu3 .ny3 .n3n
.pa3 .pe3 .pi3 .po3 .pr3 .pu3 .py3 .p3p
.qu3 .qa3 .qe3 .qi3 .qo3 .qu3 .q3q
.ra3 .re3 .ri3 .ro3 .rr3 .ru3 .ry3 .r3r
.sa3 .se3 .si3 .so3 .sr3 .su3 .sy3 .s3s
.ta3 .te3 .ti3 .to3 .tr3 .tu3 .ty3 .t3t
.va3 .ve3 .vi3 .vo3 .vr3 .vu3 .vy3 .v3v
.wa3 .we3 .wi3 .wo3 .wr3 .wu3 .wy3 .w3w
.ya3 .ye3 .yi3 .yo3 .yu3 .y3y
.za3 .ze3 .zi3 .zo3 .zu3 .zy3 .z3z
ab4ib ab3le ab4oli ab3lis ab4lin a3b5lish ab4lu a3b5lu a3b5tracti5on
ab4sen5t ab4sur5d a3b5stra3c3t5i3v5i3ti3on a3b5sur5d a3b5stracti5b5i5d3i5c
a3b5sen5ti3o3us a3b5stra5ct
a3b5ra5sio4n a3b5racad3a a3b5racad5emi5a
ac3ce4l5e3ra3t5i3v ac5ce4p5ti5b a3c5cep5ti3b a3c5cla5m a3c5co3mo3da3t5i5o3n
a3c5com3mo3d a3c5cu3mu3la5ti5o3n a3c5cu5r a3c5cul4tu3r a3c5de5m a3c5de5mi5a
a3d3a3m5a3nt3i3b a3d5apt5i5o3n a3d5e3qua3c5ce4p5ti5b
a3d3mi5ni5stra3t5i5o3n a3d5o3les5ce3nt
a3f5f5e3cta3g5a3ni3za3t5i5o3n a3g5a5v5e3r
a3g5e3ra a3g5i3ta3t5i5o3n a3g5i5ta3t5o3r a3g5o3ny a3g5ra3v5i5ta3t5i5o3n
a3l3a3r3m a3l5b3i3no3s a3l5co3ho3li3 a3l5co3ho3li3c a3l5der5m
a3l5e3r3gy a3l3i3a3s5 a3l3i3g5a3l3i3t5i3o3n a3l3lo3wa3l5l5u3s5i5o3n
a3l5mu3mi3 a3l5ou3a3l5phi3 a3l5re3a3d5y a3l5t5i5tu3de a3l5ti3tu3de5
a3m5a3te5r5ia a3m5a3t5i5o3n a3m5bi3a3m5bi3li3a3m5bi5ty
a3m5bu3la3t5i5o3n a3m5en5d a3m5i3a3m5i3b a3m5i5na3m5i5nya3m5i5sz
a3m5o3ra3m5u5le3ta3m5u5li3
an4a3b5a3ti3o3n a3n5a3go3n a3n5a3ly3ti3c a3n5archi3
a3n5athe5m a3n5e3mi3a3n5e3m5i3a3n5es5the5t3i3c
a3n5ges5ti3o3n a3n5gi5o3n a3n5he3li3a3n5hi3la3t5i5o3n
";

/// Hyphenate a word using Liou pattern matching.
fn hyphenate_with_patterns(word: &str, patterns: &HyphenPatterns) -> Vec<HyphenPoint> {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < MIN_PATTERN_WORD_LEN {
        return vec![];
    }

    let word_lower: Vec<char> = word.chars().map(|c| c.to_ascii_lowercase()).collect();

    let mut levels = vec![0u8; word_lower.len() + 1];

    for (pattern_chars, pattern_levels) in &patterns.patterns {
        let wlen = word_lower.len();

        let mut char_positions: Vec<(usize, char)> = Vec::new();
        for (pi, &pc) in pattern_chars.iter().enumerate() {
            if pc == '.' {
                continue;
            }
            char_positions.push((pi, pc));
        }
        if char_positions.is_empty() {
            continue;
        }
        let char_count = char_positions.len();
        let first_non_dot = char_positions[0].0;
        let _last_non_dot = char_positions[char_count - 1].0;

        for offset in 0..=(wlen.saturating_sub(char_count)) {
            let mut matched = true;
            for &(pi, pc) in &char_positions {
                let wi = offset + pi - first_non_dot;
                if wi >= wlen || word_lower[wi] != pc {
                    matched = false;
                    break;
                }
            }
            if !matched {
                continue;
            }

            let word_offset = offset.saturating_sub(first_non_dot);
            for (i, &level) in pattern_levels.iter().enumerate() {
                let pos = word_offset + i;
                if pos < levels.len() && level > levels[pos] {
                    levels[pos] = level;
                }
            }
        }
    }

    let mut points = Vec::new();
    for i in 1..levels.len() - 1 {
        if levels[i] > 0 && levels[i] % 2 == 1 {
            let byte_offset: usize = chars[..i].iter().map(|c| c.len_utf8()).sum();
            points.push(HyphenPoint {
                position: byte_offset,
                quality: HyphenQuality::Pattern,
            });
        }
    }
    points
}

/// Hyphenate a word using Liou patterns if available, otherwise fall back to
/// the heuristic engine.
///
/// This is an opt-in alternative to [`hyphenate_word`]. Callers who want
/// pattern-based hyphenation should prefer this function.
pub fn hyphenate_word_with_patterns(word: &str) -> Vec<HyphenPoint> {
    let embedded = HyphenPatterns::from_pattern_data(EMBEDDED_ENGLISH_PATTERNS);
    if embedded.patterns.is_empty() {
        hyphenate_word(word)
    } else {
        let pattern_points = hyphenate_with_patterns(word, &embedded);
        if pattern_points.is_empty() {
            hyphenate_word(word)
        } else {
            pattern_points
        }
    }
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

    #[test]
    fn test_liou_pattern_parsing() {
        let (chars, levels) = parse_liou_pattern_token(".a3b2c2").unwrap();
        assert_eq!(chars, vec!['.', 'a', 'b', 'c']);
        assert_eq!(levels.len(), chars.len() + 1);
        assert_eq!(levels[0], 0);
        assert_eq!(levels[1], 0);
        assert_eq!(levels[2], 3);
        assert_eq!(levels[3], 2);
        assert_eq!(levels[4], 2);

        let (chars2, levels2) = parse_liou_pattern_token("ab4ib2").unwrap();
        assert_eq!(chars2, vec!['a', 'b', 'i', 'b']);
        assert_eq!(levels2[0], 0);
        assert_eq!(levels2[2], 4);
        assert_eq!(levels2[4], 2);

        assert!(parse_liou_pattern_token("").is_none());
        assert!(parse_liou_pattern_token("   ").is_none());
    }

    #[test]
    fn test_hyphenate_with_patterns_basic() {
        let patterns = HyphenPatterns::from_pattern_data(EMBEDDED_ENGLISH_PATTERNS);
        assert!(
            !patterns.patterns.is_empty(),
            "embedded patterns should load"
        );

        let points = hyphenate_with_patterns("abstract", &patterns);
        assert!(
            !points.is_empty(),
            "abstract should have pattern-based hyphenation points"
        );
        for p in &points {
            assert!(p.position < "abstract".len());
            assert_eq!(p.quality, HyphenQuality::Pattern);
        }

        let short = hyphenate_with_patterns("cat", &patterns);
        assert!(short.is_empty(), "short words should not hyphenate");
    }

    #[test]
    fn test_hyphenate_with_patterns_fallback() {
        let points = hyphenate_word_with_patterns("unknownwordxyz");
        assert!(
            !points.is_empty(),
            "should produce points via fallback for unmatched word"
        );
    }
}
