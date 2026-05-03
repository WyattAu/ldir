//! RTL (right-to-left) text detection and bidirectional reordering.

#![allow(dead_code)]
#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

/// Direction of a text run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

/// A contiguous run of text with the same direction.
#[derive(Debug, Clone)]
pub struct DirectionRun {
    pub start: usize,
    pub end: usize,
    pub direction: TextDirection,
    pub level: u8,
}

/// Check if a character is a strong RTL character.
pub fn is_rtl_strong(ch: char) -> bool {
    matches!(
        ch,
        '\u{0590}'..='\u{05FF}'   // Hebrew
        | '\u{0600}'..='\u{06FF}' // Arabic
        | '\u{0700}'..='\u{074F}' // Syriac
        | '\u{0750}'..='\u{077F}' // Arabic Supplement
        | '\u{0780}'..='\u{07BF}' // Thaana
        | '\u{FB50}'..='\u{FDFF}' // Arabic Presentation Forms-A
        | '\u{FE70}'..='\u{FEFF}' // Arabic Presentation Forms-B
    )
}

pub fn is_rtl_char(ch: char) -> bool {
    is_rtl_strong(ch)
}

pub fn is_rtl_text(text: &str) -> bool {
    let rtl_count = text
        .chars()
        .filter(|c| is_rtl_char(*c) && !c.is_whitespace())
        .count();
    let total: usize = text.chars().filter(|c| !c.is_whitespace()).count();
    total > 0 && rtl_count as f64 / total as f64 > 0.5
}

/// Check if a character is a strong LTR character.
fn is_ltr_strong(ch: char) -> bool {
    matches!(ch, 'A'..='Z' | 'a'..='z' | '\u{00C0}'..='\u{024F}')
}

/// Determine the base direction from the first strong character.
pub fn base_direction(text: &str) -> TextDirection {
    for ch in text.chars() {
        if is_rtl_strong(ch) {
            return TextDirection::RightToLeft;
        }
        if is_ltr_strong(ch) {
            return TextDirection::LeftToRight;
        }
    }
    TextDirection::LeftToRight
}

/// Analyze text and produce direction runs in visual order.
///
/// Simplified Unicode Bidirectional Algorithm:
/// 1. Determine base direction from first strong character (P2, P3 from UAX#9).
/// 2. Walk characters, assigning embedding levels: neutral characters inherit
///    the surrounding strong direction; whitespace inherits the base direction.
/// 3. Merge adjacent runs with the same level.
/// 4. Reverse the sequence of runs at odd (RTL) embedding levels so the result
///    is in visual order.
pub fn analyze_bidi(text: &str) -> Vec<DirectionRun> {
    if text.is_empty() {
        return vec![];
    }

    let base = base_direction(text);
    let base_level: u8 = match base {
        TextDirection::LeftToRight => 0,
        TextDirection::RightToLeft => 1,
    };

    let chars: Vec<char> = text.chars().collect();
    let char_indices: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    let len = chars.len();

    // Assign an embedding level to every character.
    let mut levels: Vec<u8> = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        if is_rtl_strong(ch) {
            levels.push(1);
            i += 1;
        } else if is_ltr_strong(ch) {
            levels.push(0);
            i += 1;
        } else if ch.is_whitespace() {
            // Whitespace inherits the base level.
            levels.push(base_level);
            i += 1;
        } else if ch == '\n' || ch == '\t' {
            // Block separators and tabs inherit the base level.
            levels.push(base_level);
            i += 1;
        } else {
            // Neutral characters (punctuation, digits, etc.) — look ahead to
            // find the next strong character; if none, look back.
            let mut resolved = base_level;
            for ch in &chars[i + 1..len] {
                if is_rtl_strong(*ch) {
                    resolved = 1;
                    break;
                }
                if is_ltr_strong(*ch) {
                    resolved = 0;
                    break;
                }
            }
            if resolved == base_level && i > 0 {
                // Fall back to preceding strong character.
                for ch in chars[..i].iter().rev() {
                    if is_rtl_strong(*ch) {
                        resolved = 1;
                        break;
                    }
                    if is_ltr_strong(*ch) {
                        resolved = 0;
                        break;
                    }
                }
            }
            levels.push(resolved);
            i += 1;
        }
    }

    // Merge adjacent characters with the same level into runs.
    let mut logical_runs: Vec<DirectionRun> = Vec::new();
    if len > 0 {
        let mut start = 0;
        let mut current_level = levels[0];
        for idx in 1..len {
            if levels[idx] != current_level {
                logical_runs.push(DirectionRun {
                    start: char_indices[start],
                    end: char_indices[idx],
                    direction: if current_level.is_multiple_of(2) {
                        TextDirection::LeftToRight
                    } else {
                        TextDirection::RightToLeft
                    },
                    level: current_level,
                });
                start = idx;
                current_level = levels[idx];
            }
        }
        logical_runs.push(DirectionRun {
            start: char_indices[start],
            end: text.len(),
            direction: if current_level.is_multiple_of(2) {
                TextDirection::LeftToRight
            } else {
                TextDirection::RightToLeft
            },
            level: current_level,
        });
    }

    // Reverse runs at odd (RTL) embedding levels to produce visual order.
    // This is a simplification of rule L1/L2 from UAX#9: reverse each
    // maximal contiguous sequence of odd-level runs.
    reorder_runs_to_visual(&mut logical_runs)
}

/// Reverse maximal contiguous sequences of RTL (odd-level) runs to produce
/// visual order.
fn reorder_runs_to_visual(runs: &mut [DirectionRun]) -> Vec<DirectionRun> {
    let mut result = Vec::with_capacity(runs.len());
    let mut i = 0;
    while i < runs.len() {
        if runs[i].level % 2 == 1 {
            // Collect contiguous odd-level runs.
            let seq_start = i;
            while i < runs.len() && runs[i].level % 2 == 1 {
                i += 1;
            }
            // Reverse this sub-sequence for visual order.
            let mut seq: Vec<DirectionRun> = runs[seq_start..i].to_vec();
            seq.reverse();
            result.extend(seq);
        } else {
            result.push(runs[i].clone());
            i += 1;
        }
    }
    result
}

/// Reverse a string slice for RTL rendering (reverses grapheme clusters via
/// char iteration — good enough for Hebrew/Arabic without combining marks).
pub fn reverse_rtl_run(text: &str) -> String {
    text.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rtl_strong_hebrew() {
        assert!(is_rtl_strong('\u{05D0}')); // Alef
        assert!(is_rtl_strong('\u{05EA}')); // Tav
        assert!(is_rtl_strong('\u{0590}')); // first Hebrew range
        assert!(is_rtl_strong('\u{05FF}')); // last Hebrew range
    }

    #[test]
    fn test_is_rtl_strong_arabic() {
        assert!(is_rtl_strong('\u{0627}')); // Alef
        assert!(is_rtl_strong('\u{0628}')); // Ba
        assert!(is_rtl_strong('\u{0600}')); // first Arabic range
        assert!(is_rtl_strong('\u{06FF}')); // last Arabic range
    }

    #[test]
    fn test_is_rtl_strong_latin_false() {
        assert!(!is_rtl_strong('a'));
        assert!(!is_rtl_strong('Z'));
        assert!(!is_rtl_strong('0'));
        assert!(!is_rtl_strong(' '));
        assert!(!is_rtl_strong('.'));
    }

    #[test]
    fn test_analyze_bidi_ltr() {
        let runs = analyze_bidi("Hello World");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, TextDirection::LeftToRight);
        assert_eq!(runs[0].level, 0);
    }

    #[test]
    fn test_analyze_bidi_rtl() {
        let runs = analyze_bidi("שלום עולם");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].direction, TextDirection::RightToLeft);
        assert_eq!(runs[0].level, 1);
    }

    #[test]
    fn test_analyze_bidi_mixed() {
        // "Hello שלום World" — LTR base with embedded RTL.
        let text = "Hello שלום World";
        let runs = analyze_bidi(text);
        // Should have at least 3 runs: LTR, RTL, LTR
        assert!(runs.len() >= 3);
        assert_eq!(runs[0].direction, TextDirection::LeftToRight);
        // Find an RTL run
        assert!(runs.iter().any(|r| r.direction == TextDirection::RightToLeft));
        assert_eq!(runs[runs.len() - 1].direction, TextDirection::LeftToRight);
    }

    #[test]
    fn test_reverse_rtl_run() {
        let original = "ABC";
        let reversed = reverse_rtl_run(original);
        assert_eq!(reversed, "CBA");
    }

    #[test]
    fn test_reverse_rtl_run_preserves_chars() {
        let original = "שלום";
        let reversed = reverse_rtl_run(original);
        // Each Hebrew letter is a single char, so reversing chars gives visual order.
        assert_eq!(reversed.chars().count(), original.chars().count());
        assert_eq!(reversed, "םולש");
    }

    #[test]
    fn test_base_direction_detection_ltr() {
        assert_eq!(base_direction("Hello"), TextDirection::LeftToRight);
        assert_eq!(base_direction("123 Hello"), TextDirection::LeftToRight);
    }

    #[test]
    fn test_base_direction_detection_rtl() {
        assert_eq!(base_direction("שלום"), TextDirection::RightToLeft);
        assert_eq!(base_direction("مرحبا"), TextDirection::RightToLeft);
    }

    #[test]
    fn test_base_direction_mixed_first_strong_wins() {
        // First strong character is Latin.
        assert_eq!(base_direction("Hello שלום"), TextDirection::LeftToRight);
        // First strong character is Hebrew.
        assert_eq!(base_direction("שלום Hello"), TextDirection::RightToLeft);
    }

    #[test]
    fn test_base_direction_neutral_only() {
        // Only digits and whitespace — defaults to LTR.
        assert_eq!(base_direction("123 456"), TextDirection::LeftToRight);
    }

    #[test]
    fn test_analyze_bidi_empty() {
        let runs = analyze_bidi("");
        assert!(runs.is_empty());
    }

    #[test]
    fn test_analyze_bidi_rtl_base_mixed() {
        // RTL base with embedded LTR.
        let text = "שלום Hello עולם";
        let runs = analyze_bidi(text);
        assert!(runs.len() >= 3);
        assert_eq!(runs[0].direction, TextDirection::RightToLeft);
        assert!(runs.iter().any(|r| r.direction == TextDirection::LeftToRight));
    }

    #[test]
    fn test_is_rtl_char_hebrew() {
        assert!(is_rtl_char('\u{05D0}')); // Alef
        assert!(is_rtl_char('\u{05EA}')); // Tav
        assert!(is_rtl_char('\u{0590}')); // first Hebrew range
        assert!(is_rtl_char('\u{05FF}')); // last Hebrew range
    }

    #[test]
    fn test_is_rtl_char_arabic() {
        assert!(is_rtl_char('\u{0627}')); // Alef
        assert!(is_rtl_char('\u{0628}')); // Ba
        assert!(is_rtl_char('\u{0600}')); // first Arabic range
        assert!(is_rtl_char('\u{06FF}')); // last Arabic range
    }

    #[test]
    fn test_is_rtl_char_syriac() {
        assert!(is_rtl_char('\u{0710}'));
        assert!(is_rtl_char('\u{074F}'));
    }

    #[test]
    fn test_is_rtl_char_arabic_supplement() {
        assert!(is_rtl_char('\u{0750}'));
        assert!(is_rtl_char('\u{077F}'));
    }

    #[test]
    fn test_is_rtl_char_presentation_forms() {
        assert!(is_rtl_char('\u{FB50}'));
        assert!(is_rtl_char('\u{FE70}'));
    }

    #[test]
    fn test_is_rtl_char_latin_false() {
        assert!(!is_rtl_char('a'));
        assert!(!is_rtl_char('Z'));
        assert!(!is_rtl_char('0'));
        assert!(!is_rtl_char(' '));
    }

    #[test]
    fn test_is_rtl_char_cjk_false() {
        assert!(!is_rtl_char('你'));
        assert!(!is_rtl_char('あ'));
    }

    #[test]
    fn test_is_rtl_char_boundaries() {
        assert!(!is_rtl_char('\u{058F}')); // just before Hebrew
        assert!(is_rtl_char('\u{0590}')); // first Hebrew
        assert!(is_rtl_char('\u{05FF}')); // last Hebrew
        assert!(is_rtl_char('\u{0600}')); // first Arabic
        assert!(!is_rtl_char('\u{0800}')); // just after Arabic Supplement
    }

    #[test]
    fn test_is_rtl_text_pure_hebrew() {
        assert!(is_rtl_text("שלום עולם"));
    }

    #[test]
    fn test_is_rtl_text_pure_arabic() {
        assert!(is_rtl_text("مرحبا بالعالم"));
    }

    #[test]
    fn test_is_rtl_text_latin_false() {
        assert!(!is_rtl_text("Hello World"));
    }

    #[test]
    fn test_is_rtl_text_empty() {
        assert!(!is_rtl_text(""));
    }

    #[test]
    fn test_is_rtl_text_whitespace_only() {
        assert!(!is_rtl_text("   "));
    }

    #[test]
    fn test_is_rtl_text_mixed_rtl_dominant() {
        assert!(is_rtl_text("שלום Hello עולם"));
    }

    #[test]
    fn test_is_rtl_text_mixed_ltr_dominant() {
        assert!(!is_rtl_text("Hello שלום World Test"));
    }

    #[test]
    fn test_is_rtl_text_exactly_half() {
        let text = "שa"; // 1 RTL, 1 LTR = 50%
        assert!(!is_rtl_text(text));
    }

    #[test]
    fn test_is_rtl_text_slightly_over_half() {
        let text = "שלa"; // 2 RTL, 1 LTR = 66.7%
        assert!(is_rtl_text(text));
    }
}
