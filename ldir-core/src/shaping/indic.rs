//! Indic text shaping support.
//!
//! Provides script detection, character classification, syllable clustering,
//! and line-break analysis for the ten major Indic scripts defined in Unicode.

#![deny(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicScript {
    Devanagari,
    Bengali,
    Gurmukhi,
    Gujarati,
    Oriya,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Sinhala,
}

pub fn is_indic_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{0900}'..='\u{097F}'
            | '\u{0980}'..='\u{09FF}'
            | '\u{0A00}'..='\u{0A7F}'
            | '\u{0A80}'..='\u{0AFF}'
            | '\u{0B00}'..='\u{0B7F}'
            | '\u{0B80}'..='\u{0BFF}'
            | '\u{0C00}'..='\u{0C7F}'
            | '\u{0C80}'..='\u{0CFF}'
            | '\u{0D00}'..='\u{0D7F}'
            | '\u{0D80}'..='\u{0DFF}'
    )
}

pub fn detect_indic_script(ch: char) -> Option<IndicScript> {
    match ch {
        '\u{0900}'..='\u{097F}' => Some(IndicScript::Devanagari),
        '\u{0980}'..='\u{09FF}' => Some(IndicScript::Bengali),
        '\u{0A00}'..='\u{0A7F}' => Some(IndicScript::Gurmukhi),
        '\u{0A80}'..='\u{0AFF}' => Some(IndicScript::Gujarati),
        '\u{0B00}'..='\u{0B7F}' => Some(IndicScript::Oriya),
        '\u{0B80}'..='\u{0BFF}' => Some(IndicScript::Tamil),
        '\u{0C00}'..='\u{0C7F}' => Some(IndicScript::Telugu),
        '\u{0C80}'..='\u{0CFF}' => Some(IndicScript::Kannada),
        '\u{0D00}'..='\u{0D7F}' => Some(IndicScript::Malayalam),
        '\u{0D80}'..='\u{0DFF}' => Some(IndicScript::Sinhala),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicCharCategory {
    Consonant,
    Vowel,
    VowelSign,
    Virama,
    Nukta,
    Danda,
    Digit,
    Other,
}

pub fn indic_char_category(ch: char) -> IndicCharCategory {
    let cp = ch as u32;

    if cp == 0x0964 || cp == 0x0965 {
        return IndicCharCategory::Danda;
    }

    if matches!(
        cp,
        0x094D | 0x09CD | 0x0A4D | 0x0ACD | 0x0B4D | 0x0BCD | 0x0C4D | 0x0CCD | 0x0D4D | 0x0DCA
    ) {
        return IndicCharCategory::Virama;
    }

    if matches!(
        cp,
        0x093C | 0x09BC | 0x0A3C | 0x0ABC | 0x0B3C | 0x0C3C | 0x0CBC | 0x0D3C
    ) {
        return IndicCharCategory::Nukta;
    }

    if matches!(
        cp,
        0x0966..=0x096F
            | 0x09E6..=0x09EF
            | 0x0A66..=0x0A6F
            | 0x0AE6..=0x0AEF
            | 0x0B66..=0x0B6F
            | 0x0BE6..=0x0BEF
            | 0x0C66..=0x0C6F
            | 0x0CE6..=0x0CEF
            | 0x0D66..=0x0D6F
            | 0x0DE6..=0x0DEF
    ) {
        return IndicCharCategory::Digit;
    }

    classify_by_script(cp)
}

fn classify_by_script(cp: u32) -> IndicCharCategory {
    match cp {
        0x0900..=0x097F => classify_devanagari(cp),
        0x0980..=0x09FF => classify_bengali(cp),
        0x0A00..=0x0A7F => classify_gurmukhi(cp),
        0x0A80..=0x0AFF => classify_gujarati(cp),
        0x0B00..=0x0B7F => classify_oriya(cp),
        0x0B80..=0x0BFF => classify_tamil(cp),
        0x0C00..=0x0C7F => classify_telugu(cp),
        0x0C80..=0x0CFF => classify_kannada(cp),
        0x0D00..=0x0D7F => classify_malayalam(cp),
        0x0D80..=0x0DFF => classify_sinhala(cp),
        _ => IndicCharCategory::Other,
    }
}

fn in_ranges(cp: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|&(lo, hi)| (lo..=hi).contains(&cp))
}

fn classify_devanagari(cp: u32) -> IndicCharCategory {
    if in_ranges(cp, &[(0x0915, 0x0939), (0x0958, 0x095F)]) {
        IndicCharCategory::Consonant
    } else if in_ranges(cp, &[(0x0904, 0x0914), (0x0960, 0x0961)]) {
        IndicCharCategory::Vowel
    } else if in_ranges(cp, &[(0x093E, 0x094C), (0x0962, 0x0963)]) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_bengali(cp: u32) -> IndicCharCategory {
    if in_ranges(cp, &[(0x0995, 0x09B9)]) {
        IndicCharCategory::Consonant
    } else if in_ranges(cp, &[(0x0985, 0x0994), (0x09E0, 0x09E1)]) {
        IndicCharCategory::Vowel
    } else if in_ranges(cp, &[(0x09BE, 0x09CC), (0x09D7, 0x09D7), (0x09E2, 0x09E3)]) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_gurmukhi(cp: u32) -> IndicCharCategory {
    if in_ranges(
        cp,
        &[
            (0x0A15, 0x0A28),
            (0x0A2A, 0x0A30),
            (0x0A32, 0x0A33),
            (0x0A35, 0x0A36),
            (0x0A38, 0x0A39),
            (0x0A59, 0x0A5E),
        ],
    ) {
        IndicCharCategory::Consonant
    } else if in_ranges(
        cp,
        &[
            (0x0A05, 0x0A0A),
            (0x0A0F, 0x0A10),
            (0x0A13, 0x0A14),
            (0x0A72, 0x0A74),
        ],
    ) {
        IndicCharCategory::Vowel
    } else if in_ranges(cp, &[(0x0A3E, 0x0A42), (0x0A47, 0x0A48), (0x0A4B, 0x0A4C)]) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_gujarati(cp: u32) -> IndicCharCategory {
    if in_ranges(
        cp,
        &[
            (0x0A95, 0x0AA8),
            (0x0AAA, 0x0AB0),
            (0x0AB2, 0x0AB3),
            (0x0AB5, 0x0AB9),
        ],
    ) {
        IndicCharCategory::Consonant
    } else if in_ranges(
        cp,
        &[
            (0x0A85, 0x0A8B),
            (0x0A8D, 0x0A8D),
            (0x0A8F, 0x0A91),
            (0x0A93, 0x0A94),
        ],
    ) {
        IndicCharCategory::Vowel
    } else if in_ranges(
        cp,
        &[
            (0x0ABE, 0x0AC0),
            (0x0AC1, 0x0AC5),
            (0x0AC7, 0x0AC9),
            (0x0ACB, 0x0ACC),
            (0x0AE0, 0x0AE1),
        ],
    ) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_oriya(cp: u32) -> IndicCharCategory {
    if in_ranges(cp, &[(0x0B15, 0x0B39)]) {
        IndicCharCategory::Consonant
    } else if in_ranges(cp, &[(0x0B05, 0x0B0C), (0x0B0F, 0x0B10), (0x0B13, 0x0B14)]) {
        IndicCharCategory::Vowel
    } else if in_ranges(
        cp,
        &[
            (0x0B3E, 0x0B43),
            (0x0B47, 0x0B48),
            (0x0B4B, 0x0B4C),
            (0x0B56, 0x0B57),
        ],
    ) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_tamil(cp: u32) -> IndicCharCategory {
    if in_ranges(
        cp,
        &[
            (0x0B95, 0x0B9A),
            (0x0B9C, 0x0B9C),
            (0x0B9E, 0x0B9F),
            (0x0BA3, 0x0BA4),
            (0x0BA8, 0x0BA9),
            (0x0BAE, 0x0BB9),
        ],
    ) {
        IndicCharCategory::Consonant
    } else if in_ranges(cp, &[(0x0B85, 0x0B8A), (0x0B8E, 0x0B90), (0x0B92, 0x0B95)]) {
        IndicCharCategory::Vowel
    } else if in_ranges(
        cp,
        &[
            (0x0BBE, 0x0BC2),
            (0x0BC6, 0x0BC8),
            (0x0BCA, 0x0BCC),
            (0x0BD7, 0x0BD7),
        ],
    ) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_telugu(cp: u32) -> IndicCharCategory {
    if in_ranges(cp, &[(0x0C15, 0x0C28), (0x0C2A, 0x0C39)]) {
        IndicCharCategory::Consonant
    } else if in_ranges(cp, &[(0x0C05, 0x0C0C), (0x0C0E, 0x0C10), (0x0C12, 0x0C14)]) {
        IndicCharCategory::Vowel
    } else if in_ranges(
        cp,
        &[
            (0x0C3E, 0x0C44),
            (0x0C46, 0x0C48),
            (0x0C4A, 0x0C4C),
            (0x0C55, 0x0C56),
        ],
    ) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_kannada(cp: u32) -> IndicCharCategory {
    if in_ranges(cp, &[(0x0C95, 0x0CA8), (0x0CAA, 0x0CB3), (0x0CB5, 0x0CB9)]) {
        IndicCharCategory::Consonant
    } else if in_ranges(cp, &[(0x0C85, 0x0C8C), (0x0C8E, 0x0C90), (0x0C92, 0x0C94)]) {
        IndicCharCategory::Vowel
    } else if in_ranges(
        cp,
        &[
            (0x0CBE, 0x0CC4),
            (0x0CC6, 0x0CC8),
            (0x0CCA, 0x0CCC),
            (0x0CD5, 0x0CD6),
        ],
    ) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_malayalam(cp: u32) -> IndicCharCategory {
    if in_ranges(cp, &[(0x0D15, 0x0D3A)]) {
        IndicCharCategory::Consonant
    } else if in_ranges(cp, &[(0x0D05, 0x0D0C), (0x0D0E, 0x0D10), (0x0D12, 0x0D14)]) {
        IndicCharCategory::Vowel
    } else if in_ranges(
        cp,
        &[
            (0x0D3E, 0x0D44),
            (0x0D46, 0x0D48),
            (0x0D4A, 0x0D4C),
            (0x0D57, 0x0D57),
        ],
    ) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

fn classify_sinhala(cp: u32) -> IndicCharCategory {
    if in_ranges(
        cp,
        &[
            (0x0D9A, 0x0DB1),
            (0x0DB3, 0x0DBB),
            (0x0DBD, 0x0DBD),
            (0x0DC0, 0x0DC6),
        ],
    ) {
        IndicCharCategory::Consonant
    } else if in_ranges(cp, &[(0x0D85, 0x0D96)]) {
        IndicCharCategory::Vowel
    } else if in_ranges(
        cp,
        &[
            (0x0DCF, 0x0DD4),
            (0x0DD6, 0x0DD6),
            (0x0DD8, 0x0DDF),
            (0x0DF2, 0x0DF3),
        ],
    ) {
        IndicCharCategory::VowelSign
    } else {
        IndicCharCategory::Other
    }
}

#[derive(Debug, Clone)]
pub struct IndicCluster {
    pub chars: Vec<char>,
    pub start: usize,
    pub end: usize,
}

pub fn cluster_indic_text(text: &str) -> Vec<IndicCluster> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut clusters = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let (start, ch) = chars[i];
        let cat = indic_char_category(ch);
        let mut cluster_chars = vec![ch];
        let mut j = i + 1;

        match cat {
            IndicCharCategory::Consonant => loop {
                if j >= chars.len() {
                    break;
                }
                let next_cat = indic_char_category(chars[j].1);
                match next_cat {
                    IndicCharCategory::Nukta => {
                        cluster_chars.push(chars[j].1);
                        j += 1;
                    }
                    IndicCharCategory::Virama => {
                        cluster_chars.push(chars[j].1);
                        j += 1;
                        if j < chars.len()
                            && indic_char_category(chars[j].1) == IndicCharCategory::Consonant
                        {
                            cluster_chars.push(chars[j].1);
                            j += 1;
                            continue;
                        }
                        break;
                    }
                    IndicCharCategory::VowelSign => {
                        cluster_chars.push(chars[j].1);
                        j += 1;
                        break;
                    }
                    _ => break,
                }
            },
            IndicCharCategory::Vowel | IndicCharCategory::Danda | IndicCharCategory::Other => {
                j = i + 1;
            }
            IndicCharCategory::Digit => {
                while j < chars.len() && indic_char_category(chars[j].1) == IndicCharCategory::Digit
                {
                    cluster_chars.push(chars[j].1);
                    j += 1;
                }
            }
            IndicCharCategory::VowelSign | IndicCharCategory::Virama | IndicCharCategory::Nukta => {
                j = i + 1;
            }
        }

        let end = if j < chars.len() {
            chars[j].0
        } else {
            text.len()
        };
        clusters.push(IndicCluster {
            chars: cluster_chars,
            start,
            end,
        });
        i = j;
    }

    clusters
}

pub fn indic_break_allowed(text: &str, index: usize) -> bool {
    if index == 0 || index == text.len() {
        return true;
    }
    if !text.is_char_boundary(index) {
        return false;
    }
    let clusters = cluster_indic_text(text);
    clusters.iter().any(|c| c.start == index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_indic_char_devanagari() {
        assert!(is_indic_char('क'));
        assert!(is_indic_char('\u{0900}'));
        assert!(is_indic_char('\u{097F}'));
    }

    #[test]
    fn is_indic_char_bengali() {
        assert!(is_indic_char('অ'));
        assert!(is_indic_char('\u{0980}'));
        assert!(is_indic_char('\u{09FF}'));
    }

    #[test]
    fn is_indic_char_tamil() {
        assert!(is_indic_char('த'));
        assert!(is_indic_char('\u{0B80}'));
        assert!(is_indic_char('\u{0BFF}'));
    }

    #[test]
    fn is_indic_char_returns_false_latin() {
        assert!(!is_indic_char('A'));
        assert!(!is_indic_char('z'));
        assert!(!is_indic_char('0'));
        assert!(!is_indic_char(' '));
    }

    #[test]
    fn detect_indic_script_devanagari() {
        assert_eq!(detect_indic_script('क'), Some(IndicScript::Devanagari));
        assert_eq!(
            detect_indic_script('\u{094D}'),
            Some(IndicScript::Devanagari)
        );
    }

    #[test]
    fn detect_indic_script_unknown() {
        assert_eq!(detect_indic_script('A'), None);
        assert_eq!(detect_indic_script('日'), None);
        assert_eq!(detect_indic_script(' '), None);
    }

    #[test]
    fn indic_char_category_virama() {
        assert_eq!(indic_char_category('\u{094D}'), IndicCharCategory::Virama);
        assert_eq!(indic_char_category('\u{09CD}'), IndicCharCategory::Virama);
        assert_eq!(indic_char_category('\u{0BCD}'), IndicCharCategory::Virama);
    }

    #[test]
    fn indic_char_category_nukta() {
        assert_eq!(indic_char_category('\u{093C}'), IndicCharCategory::Nukta);
        assert_eq!(indic_char_category('\u{09BC}'), IndicCharCategory::Nukta);
    }

    #[test]
    fn indic_char_category_danda() {
        assert_eq!(indic_char_category('\u{0964}'), IndicCharCategory::Danda);
        assert_eq!(indic_char_category('\u{0965}'), IndicCharCategory::Danda);
    }

    #[test]
    fn cluster_indic_simple() {
        let clusters = cluster_indic_text("क");
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].chars, vec!['क']);
        assert_eq!(clusters[0].start, 0);
        assert_eq!(clusters[0].end, 'क'.len_utf8());
    }

    #[test]
    fn cluster_indic_conjunct() {
        let clusters = cluster_indic_text("क्ष");
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].chars, vec!['क', '्', 'ष']);
    }

    #[test]
    fn cluster_indic_mixed() {
        let text = "कAத";
        let clusters = cluster_indic_text(text);
        assert_eq!(clusters.len(), 3);
        assert_eq!(clusters[0].chars, vec!['क']);
        assert_eq!(clusters[1].chars, vec!['A']);
        assert_eq!(clusters[2].chars, vec!['த']);
    }

    #[test]
    fn indic_break_after_danda() {
        let text = "है।है";
        let danda_end = "है।".len();
        assert!(indic_break_allowed(text, danda_end));
    }

    #[test]
    fn indic_break_within_cluster_forbidden() {
        let text = "क्ष";
        let virama_start = 'क'.len_utf8();
        assert!(!indic_break_allowed(text, virama_start));
    }
}
