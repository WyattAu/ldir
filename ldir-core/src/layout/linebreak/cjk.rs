//! CJK-aware line breaking.
//!
//! Extends the Knuth-Plass algorithm with CJK-specific break opportunities:
//! - Any CJK character is a potential break point (penalty 0)
//! - Certain characters are prohibited at line start/end

#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

use crate::fp266::Fp266;

use super::types::LineBreakItem;

#[inline]
pub fn is_cjk_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}' // CJK Unified Ideographs Extension A
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
        | '\u{AC00}'..='\u{D7AF}' // Hangul Syllables
        | '\u{3000}'..='\u{303F}' // CJK Symbols and Punctuation
        | '\u{FF00}'..='\u{FFEF}' // Fullwidth Forms
    )
}

pub fn is_cjk_text(text: &str) -> bool {
    text.chars().any(is_cjk_char)
}

#[inline]
pub fn is_prohibited_at_line_start(ch: char) -> bool {
    matches!(
        ch,
        '，' | '。'
            | '、'
            | '：'
            | '；'
            | '！'
            | '？'
            | '〜'
            | '～'
            | '）'
            | '】'
            | '」'
            | '』'
            | '》'
            | ','
            | '.'
            | ':'
            | ';'
            | '!'
            | '?'
            | ')'
            | ']'
            | '}'
    )
}

#[inline]
pub fn is_prohibited_at_line_end(ch: char) -> bool {
    matches!(ch, '（' | '【' | '「' | '『' | '《' | '(' | '[' | '{')
}

pub fn insert_cjk_breaks(text: &str, items: &[LineBreakItem]) -> Vec<LineBreakItem> {
    if !is_cjk_text(text) || items.is_empty() {
        return items.to_vec();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() != items.len() {
        return items.to_vec();
    }

    let mut result = Vec::with_capacity(items.len() * 2);
    for (i, &item) in items.iter().enumerate() {
        result.push(item);
        if i + 1 < chars.len() {
            let curr = chars[i];
            let next = chars[i + 1];
            if is_cjk_char(curr)
                && is_cjk_char(next)
                && !is_prohibited_at_line_end(curr)
                && !is_prohibited_at_line_start(next)
            {
                result.push(LineBreakItem {
                    width: Fp266::ZERO,
                    stretchability: Fp266::ZERO,
                    shrinkability: Fp266::ZERO,
                    penalty: 0.0,
                    is_mandatory: false,
                    is_hyphenation: false,
                    hyphen_width: Fp266::ZERO,
                    text: "",
                });
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fp266::Fp266;

    fn item(width: i32) -> LineBreakItem {
        LineBreakItem {
            width: Fp266::from_int(width),
            stretchability: Fp266::ZERO,
            shrinkability: Fp266::ZERO,
            penalty: 0.0,
            is_mandatory: false,
            is_hyphenation: false,
            hyphen_width: Fp266::ZERO,
            text: "",
        }
    }

    #[test]
    fn test_is_cjk_char_chinese() {
        assert!(is_cjk_char('你'));
        assert!(is_cjk_char('好'));
        assert!(is_cjk_char('世'));
        assert!(is_cjk_char('界'));
    }

    #[test]
    fn test_is_cjk_char_japanese_hiragana() {
        assert!(is_cjk_char('あ'));
        assert!(is_cjk_char('い'));
        assert!(is_cjk_char('う'));
    }

    #[test]
    fn test_is_cjk_char_japanese_katakana() {
        assert!(is_cjk_char('ア'));
        assert!(is_cjk_char('イ'));
        assert!(is_cjk_char('ウ'));
    }

    #[test]
    fn test_is_cjk_char_korean() {
        assert!(is_cjk_char('한'));
        assert!(is_cjk_char('국'));
    }

    #[test]
    fn test_is_cjk_char_cjk_punctuation() {
        assert!(is_cjk_char('、'));
        assert!(is_cjk_char('。'));
        assert!(is_cjk_char('「'));
        assert!(is_cjk_char('」'));
    }

    #[test]
    fn test_is_cjk_char_fullwidth_forms() {
        assert!(is_cjk_char('，'));
        assert!(is_cjk_char('Ａ'));
        assert!(is_cjk_char('（'));
    }

    #[test]
    fn test_is_cjk_char_latin_false() {
        assert!(!is_cjk_char('a'));
        assert!(!is_cjk_char('Z'));
        assert!(!is_cjk_char('0'));
        assert!(!is_cjk_char(' '));
        assert!(!is_cjk_char('\n'));
    }

    #[test]
    fn test_is_cjk_char_cjk_extension_a() {
        assert!(is_cjk_char('\u{3400}'));
        assert!(is_cjk_char('\u{4DBF}'));
    }

    #[test]
    fn test_is_cjk_char_boundaries() {
        assert!(!is_cjk_char('\u{4DFF}')); // just before CJK Unified
        assert!(is_cjk_char('\u{4E00}')); // first CJK Unified
        assert!(is_cjk_char('\u{9FFF}')); // last CJK Unified
        assert!(!is_cjk_char('\u{A000}')); // just after
    }

    #[test]
    fn test_is_cjk_text_pure_cjk() {
        assert!(is_cjk_text("你好世界"));
    }

    #[test]
    fn test_is_cjk_text_mixed() {
        assert!(is_cjk_text("Hello世界"));
        assert!(is_cjk_text("テストtest"));
    }

    #[test]
    fn test_is_cjk_text_no_cjk() {
        assert!(!is_cjk_text("Hello World"));
        assert!(!is_cjk_text("12345"));
        assert!(!is_cjk_text(""));
    }

    #[test]
    fn test_prohibited_at_line_start_chinese_punctuation() {
        assert!(is_prohibited_at_line_start('，'));
        assert!(is_prohibited_at_line_start('。'));
        assert!(is_prohibited_at_line_start('、'));
        assert!(is_prohibited_at_line_start('：'));
        assert!(is_prohibited_at_line_start('；'));
        assert!(is_prohibited_at_line_start('！'));
        assert!(is_prohibited_at_line_start('？'));
    }

    #[test]
    fn test_prohibited_at_line_start_close_brackets() {
        assert!(is_prohibited_at_line_start('）'));
        assert!(is_prohibited_at_line_start('】'));
        assert!(is_prohibited_at_line_start('」'));
        assert!(is_prohibited_at_line_start('』'));
        assert!(is_prohibited_at_line_start('》'));
    }

    #[test]
    fn test_prohibited_at_line_start_latin() {
        assert!(is_prohibited_at_line_start(','));
        assert!(is_prohibited_at_line_start('.'));
        assert!(is_prohibited_at_line_start(':'));
        assert!(is_prohibited_at_line_start(';'));
        assert!(is_prohibited_at_line_start('!'));
        assert!(is_prohibited_at_line_start('?'));
        assert!(is_prohibited_at_line_start(')'));
        assert!(is_prohibited_at_line_start(']'));
        assert!(is_prohibited_at_line_start('}'));
    }

    #[test]
    fn test_prohibited_at_line_start_not_prohibited() {
        assert!(!is_prohibited_at_line_start('你'));
        assert!(!is_prohibited_at_line_start('（'));
        assert!(!is_prohibited_at_line_start('a'));
        assert!(!is_prohibited_at_line_start(' '));
    }

    #[test]
    fn test_prohibited_at_line_end_open_brackets() {
        assert!(is_prohibited_at_line_end('（'));
        assert!(is_prohibited_at_line_end('【'));
        assert!(is_prohibited_at_line_end('「'));
        assert!(is_prohibited_at_line_end('『'));
        assert!(is_prohibited_at_line_end('《'));
    }

    #[test]
    fn test_prohibited_at_line_end_latin() {
        assert!(is_prohibited_at_line_end('('));
        assert!(is_prohibited_at_line_end('['));
        assert!(is_prohibited_at_line_end('{'));
    }

    #[test]
    fn test_prohibited_at_line_end_not_prohibited() {
        assert!(!is_prohibited_at_line_end('你'));
        assert!(!is_prohibited_at_line_end('）'));
        assert!(!is_prohibited_at_line_end('a'));
        assert!(!is_prohibited_at_line_end(' '));
    }

    #[test]
    fn test_insert_cjk_breaks_basic() {
        let text = "你好世界";
        let items = vec![item(10), item(10), item(10), item(10)];
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result.len(), 7); // 4 items + 3 breaks
    }

    #[test]
    fn test_insert_cjk_breaks_no_cjk() {
        let text = "Hello";
        let items = vec![item(7), item(7), item(7), item(7), item(7)];
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_insert_cjk_breaks_with_prohibited() {
        // 你(CJK) 好(CJK) ，(CJK,prohibited_start) 世(CJK) 界(CJK)
        // Breaks: 你|好, ，|世, 世|界 (， prohibited at start blocks 好|，,
        // but ， CAN end a line so ，|世 is allowed)
        let text = "你好，世界";
        let items = vec![item(10), item(10), item(10), item(10), item(10)];
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result.len(), 8); // 5 items + 3 breaks
    }

    #[test]
    fn test_insert_cjk_breaks_open_bracket() {
        // 你(CJK) （(CJK,prohibited_end) 世(CJK) 界(CJK)
        // Breaks: 你|（ (（ not prohibited at start), 世|界
        // No break: （|世 （is prohibited at line end)
        let text = "你（世界";
        let items = vec![item(10), item(10), item(10), item(10)];
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result.len(), 6); // 4 items + 2 breaks
    }

    #[test]
    fn test_insert_cjk_breaks_empty() {
        let text = "";
        let items: Vec<LineBreakItem> = vec![];
        let result = insert_cjk_breaks(text, &items);
        assert!(result.is_empty());
    }

    #[test]
    fn test_insert_cjk_breaks_single_char() {
        let text = "你";
        let items = vec![item(10)];
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_insert_cjk_breaks_mismatched_lengths() {
        let text = "你好";
        let items = vec![item(10)]; // 1 item vs 2 chars (e.g., ligature)
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result.len(), 1); // No breaks when lengths don't match
    }

    #[test]
    fn test_insert_cjk_breaks_mixed_cjk_latin() {
        // Hello(CJK=false) 你(CJK=true) 好(CJK=true)
        // Break: 你|好 only
        let text = "Hello你好";
        let items = vec![
            item(7),
            item(7),
            item(7),
            item(7),
            item(7),
            item(10),
            item(10),
        ];
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result.len(), 8); // 7 items + 1 break
    }

    #[test]
    fn test_insert_cjk_breaks_break_items_are_zero_width() {
        let text = "你好";
        let items = vec![item(10), item(10)];
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].width, Fp266::from_int(10));
        assert_eq!(result[1].width, Fp266::ZERO);
        assert_eq!(result[2].width, Fp266::from_int(10));
    }

    #[test]
    fn test_insert_cjk_breaks_preserves_original_items() {
        let text = "你好";
        let items = vec![item(10), item(10)];
        let result = insert_cjk_breaks(text, &items);
        assert_eq!(result[0].width, items[0].width);
        assert_eq!(result[2].width, items[1].width);
    }
}
