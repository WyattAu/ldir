//! RTL (right-to-left) text detection.

#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

pub fn is_rtl_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{0590}'..='\u{05FF}' // Hebrew
        | '\u{0600}'..='\u{06FF}' // Arabic
        | '\u{0700}'..='\u{074F}' // Syriac
        | '\u{0750}'..='\u{077F}' // Arabic Supplement
        | '\u{FB50}'..='\u{FDFF}' // Arabic Presentation Forms A
        | '\u{FE70}'..='\u{FEFF}' // Arabic Presentation Forms B
    )
}

pub fn is_rtl_text(text: &str) -> bool {
    let rtl_count = text
        .chars()
        .filter(|c| is_rtl_char(*c) && !c.is_whitespace())
        .count();
    let total: usize = text.chars().filter(|c| !c.is_whitespace()).count();
    total > 0 && rtl_count as f64 / total as f64 > 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Exactly 50% RTL should NOT be considered RTL (> 0.5, not >= 0.5)
        let text = "שa"; // 1 RTL, 1 LTR = 50%
        assert!(!is_rtl_text(text));
    }

    #[test]
    fn test_is_rtl_text_slightly_over_half() {
        let text = "שלa"; // 2 RTL, 1 LTR = 66.7%
        assert!(is_rtl_text(text));
    }
}
