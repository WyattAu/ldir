//! Page number formatting: arabic, roman, alphabetic.

/// Page number style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageNumberStyle {
    /// 1, 2, 3, ...
    #[default]
    Arabic,
    /// i, ii, iii, iv, ...
    LowerRoman,
    /// I, II, III, IV, ...
    UpperRoman,
    /// a, b, c, ...
    LowerAlpha,
    /// A, B, C, ...
    UpperAlpha,
    /// No page number displayed.
    None,
}

/// Format a page number (1-indexed) according to the given style.
/// Returns None if style is None or value is 0.
pub fn format_page_number(num: u32, style: PageNumberStyle) -> Option<String> {
    if num == 0 {
        return None;
    }
    match style {
        PageNumberStyle::Arabic => Some(num.to_string()),
        PageNumberStyle::LowerRoman => Some(to_lower_roman(num)),
        PageNumberStyle::UpperRoman => Some(to_upper_roman(num)),
        PageNumberStyle::LowerAlpha => to_alpha(num),
        PageNumberStyle::UpperAlpha => to_alpha(num).map(|s| s.to_uppercase()),
        PageNumberStyle::None => None,
    }
}

/// Convert a number to lowercase Roman numerals.
/// Supports values 1-3999.
pub fn to_lower_roman(n: u32) -> String {
    to_roman(n).to_lowercase()
}

/// Convert a number to uppercase Roman numerals.
/// Supports values 1-3999.
pub fn to_upper_roman(n: u32) -> String {
    to_roman(n)
}

fn to_roman(mut n: u32) -> String {
    if n == 0 || n > 3999 {
        return n.to_string();
    }
    let values = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for &(val, sym) in &values {
        while n >= val {
            result.push_str(sym);
            n -= val;
        }
    }
    result
}

/// Convert a number to alphabetic (a-z, aa-zz, aaa-zzz, ...).
/// Returns None if the value exceeds the representable range.
pub fn to_alpha(n: u32) -> Option<String> {
    if n == 0 {
        return None;
    }
    let mut n = n;
    let mut chars = Vec::new();
    loop {
        n -= 1;
        chars.push((b'a' + (n % 26) as u8) as char);
        n /= 26;
        if n == 0 {
            break;
        }
    }
    chars.reverse();
    Some(chars.into_iter().collect())
}

/// Parse total page count from a document (e.g., for "Page X of Y" formatting).
pub fn page_x_of_y(current: u32, total: u32, style: PageNumberStyle) -> Option<String> {
    let cur = format_page_number(current, style)?;
    let tot = format_page_number(total, style)?;
    Some(format!("{cur} of {tot}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic() {
        assert_eq!(
            format_page_number(1, PageNumberStyle::Arabic),
            Some("1".into())
        );
        assert_eq!(
            format_page_number(42, PageNumberStyle::Arabic),
            Some("42".into())
        );
        assert_eq!(format_page_number(0, PageNumberStyle::Arabic), None);
    }

    #[test]
    fn test_lower_roman() {
        assert_eq!(
            format_page_number(1, PageNumberStyle::LowerRoman),
            Some("i".into())
        );
        assert_eq!(
            format_page_number(4, PageNumberStyle::LowerRoman),
            Some("iv".into())
        );
        assert_eq!(
            format_page_number(9, PageNumberStyle::LowerRoman),
            Some("ix".into())
        );
        assert_eq!(
            format_page_number(42, PageNumberStyle::LowerRoman),
            Some("xlii".into())
        );
        assert_eq!(
            format_page_number(2024, PageNumberStyle::LowerRoman),
            Some("mmxxiv".into())
        );
    }

    #[test]
    fn test_upper_roman() {
        assert_eq!(
            format_page_number(1, PageNumberStyle::UpperRoman),
            Some("I".into())
        );
        assert_eq!(
            format_page_number(4, PageNumberStyle::UpperRoman),
            Some("IV".into())
        );
        assert_eq!(
            format_page_number(1994, PageNumberStyle::UpperRoman),
            Some("MCMXCIV".into())
        );
    }

    #[test]
    fn test_lower_alpha() {
        assert_eq!(
            format_page_number(1, PageNumberStyle::LowerAlpha),
            Some("a".into())
        );
        assert_eq!(
            format_page_number(26, PageNumberStyle::LowerAlpha),
            Some("z".into())
        );
        assert_eq!(
            format_page_number(27, PageNumberStyle::LowerAlpha),
            Some("aa".into())
        );
        assert_eq!(
            format_page_number(52, PageNumberStyle::LowerAlpha),
            Some("az".into())
        );
        assert_eq!(
            format_page_number(53, PageNumberStyle::LowerAlpha),
            Some("ba".into())
        );
        assert_eq!(
            format_page_number(702, PageNumberStyle::LowerAlpha),
            Some("zz".into())
        );
        assert_eq!(
            format_page_number(703, PageNumberStyle::LowerAlpha),
            Some("aaa".into())
        );
    }

    #[test]
    fn test_upper_alpha() {
        assert_eq!(
            format_page_number(1, PageNumberStyle::UpperAlpha),
            Some("A".into())
        );
        assert_eq!(
            format_page_number(27, PageNumberStyle::UpperAlpha),
            Some("AA".into())
        );
    }

    #[test]
    fn test_none_style() {
        assert_eq!(format_page_number(5, PageNumberStyle::None), None);
    }

    #[test]
    fn test_zero_returns_none() {
        for style in [
            PageNumberStyle::Arabic,
            PageNumberStyle::LowerRoman,
            PageNumberStyle::UpperRoman,
            PageNumberStyle::LowerAlpha,
            PageNumberStyle::UpperAlpha,
        ] {
            assert_eq!(format_page_number(0, style), None);
        }
    }

    #[test]
    fn test_roman_boundaries() {
        assert_eq!(
            format_page_number(3999, PageNumberStyle::UpperRoman),
            Some("MMMCMXCIX".into())
        );
        assert_eq!(
            format_page_number(4000, PageNumberStyle::UpperRoman),
            Some("4000".into())
        );
    }

    #[test]
    fn test_page_x_of_y() {
        assert_eq!(
            page_x_of_y(3, 10, PageNumberStyle::Arabic),
            Some("3 of 10".into())
        );
        assert_eq!(
            page_x_of_y(1, 5, PageNumberStyle::LowerRoman),
            Some("i of v".into())
        );
        assert_eq!(page_x_of_y(0, 5, PageNumberStyle::Arabic), None);
    }

    #[test]
    fn test_default_style_is_arabic() {
        assert_eq!(PageNumberStyle::default(), PageNumberStyle::Arabic);
    }
}
