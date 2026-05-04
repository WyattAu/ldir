//! Optical margin alignment for hanging punctuation.

#![allow(dead_code)]

/// Characters that should hang into the left margin.
pub const HANGING_CHARS_START: &[char] = &['"', '\u{201C}', '(', '[', '{', '\u{00AB}', '\u{2018}'];

/// Characters that should hang into the right margin.
pub const HANGING_CHARS_END: &[char] = &[
    '"', '\u{201D}', ')', ']', '}', '\u{00BB}', '\u{2019}', '.', ',', ';', ':', '!', '?',
];

/// Default optical margin overhang in ems.
const DEFAULT_OVERHANG_EM: f64 = 0.3;

/// Check if a line needs optical margin adjustment.
///
/// Returns `(left_adjust, right_adjust)` in points, representing how much
/// the text should overhang into the margin. Typical values are 0.2-0.5em
/// per hanging character.
///
/// # Arguments
/// * `line` - The text content of the line (trimmed of leading/trailing whitespace)
/// * `em_size` - The font size in points (used to convert em-based overhang to points)
pub fn optical_margin_adjustment(line: &str, em_size: f64) -> (f64, f64) {
    let overhang = em_size * DEFAULT_OVERHANG_EM;
    let left = if line.starts_with(|c: char| HANGING_CHARS_START.contains(&c)) {
        overhang
    } else {
        0.0
    };
    let right = if line.ends_with(|c: char| HANGING_CHARS_END.contains(&c)) {
        overhang
    } else {
        0.0
    };
    (left, right)
}

/// Returns the penalty reduction for a line that benefits from optical margins.
///
/// Lines with hanging punctuation near the margins should have reduced demerits
/// since the optical alignment improves visual appearance.
pub fn optical_margin_penalty_reduction(line: &str) -> f64 {
    let has_left = line.starts_with(|c: char| HANGING_CHARS_START.contains(&c));
    let has_right = line.ends_with(|c: char| HANGING_CHARS_END.contains(&c));
    match (has_left, has_right) {
        (true, true) => 30.0,
        (true, false) | (false, true) => 15.0,
        (false, false) => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_left_hanging_quote() {
        let (left, right) = optical_margin_adjustment("\"Hello world", 12.0);
        assert!(left > 0.0);
        assert!(right == 0.0);
        assert!((left - 3.6).abs() < 0.01);
    }

    #[test]
    fn test_right_hanging_period() {
        let (left, right) = optical_margin_adjustment("Hello world.", 12.0);
        assert!(left == 0.0);
        assert!(right > 0.0);
    }

    #[test]
    fn test_no_hanging() {
        let (left, right) = optical_margin_adjustment("Hello world", 12.0);
        assert!(left == 0.0);
        assert!(right == 0.0);
    }

    #[test]
    fn test_both_hanging() {
        let (left, right) = optical_margin_adjustment("(Hello world)", 12.0);
        assert!(left > 0.0);
        assert!(right > 0.0);
    }

    #[test]
    fn test_curly_quotes() {
        let (left, right) = optical_margin_adjustment("\u{201C}Hello\u{201D}", 12.0);
        assert!(left > 0.0);
        assert!(right > 0.0);
    }

    #[test]
    fn test_penalty_reduction_both() {
        let reduction = optical_margin_penalty_reduction("(Hello world)");
        assert!((reduction - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_penalty_reduction_one() {
        let reduction = optical_margin_penalty_reduction("\"Hello world");
        assert!((reduction - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_penalty_reduction_none() {
        let reduction = optical_margin_penalty_reduction("Hello world");
        assert!((reduction - 0.0).abs() < 0.01);
    }
}
