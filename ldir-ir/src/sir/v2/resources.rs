//! Resource declarations for S-IR v2.

use serde::{Deserialize, Serialize};

/// Font weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

/// Font style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// Where to find a font.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FontSource {
    System,
    File(String), // path to font file
    Embedded,     // font data embedded in module
}

/// Font declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontDecl {
    pub name: String,
    pub family: String,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub source: FontSource,
    pub features: Vec<String>, // OpenType features
}

/// RGB color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: Option<u8>, // alpha
}

/// Color declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDecl {
    pub name: String,
    pub value: ColorValue,
}

/// Counter formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CounterFormat {
    Arabic,         // 1, 2, 3
    RomanLower,     // i, ii, iii
    RomanUpper,     // I, II, III
    AlphaLower,     // a, b, c
    AlphaUpper,     // A, B, C
    Custom(String), // e.g., "(1)"
}

/// When to reset a counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CounterReset {
    Never,
    PerDocument,
    PerPart,
    PerChapter,
    PerSection,
    PerSubsection,
    PerPage,
}

/// Counter declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterDecl {
    pub name: String,
    pub format: CounterFormat,
    pub reset_scope: CounterReset,
}

/// Resource declarations collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceDecls {
    pub fonts: Vec<FontDecl>,
    pub colors: Vec<ColorDecl>,
    pub counters: Vec<CounterDecl>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_decl_creation() {
        let font = FontDecl {
            name: "body".to_string(),
            family: "Inter".to_string(),
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            source: FontSource::System,
            features: vec!["liga".to_string()],
        };
        assert_eq!(font.name, "body");
        assert_eq!(font.weight, FontWeight::Regular);
        assert_eq!(font.features.len(), 1);
    }

    #[test]
    fn test_color_value() {
        let c = ColorValue {
            r: 255,
            g: 0,
            b: 128,
            a: Some(200),
        };
        assert_eq!(c.r, 255);
        assert_eq!(c.a, Some(200));

        let decl = ColorDecl {
            name: "accent".to_string(),
            value: c,
        };
        assert_eq!(decl.name, "accent");
    }

    #[test]
    fn test_counter_decl() {
        let counter = CounterDecl {
            name: "section".to_string(),
            format: CounterFormat::Arabic,
            reset_scope: CounterReset::PerChapter,
        };
        assert_eq!(counter.format, CounterFormat::Arabic);
        assert_eq!(counter.reset_scope, CounterReset::PerChapter);
    }
}
