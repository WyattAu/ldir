//! Style declarations for S-IR v2.

use serde::{Deserialize, Serialize};

use crate::sir::v2::resources::FontWeight;
use crate::sir::v2::metadata::Dimension;

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
    Justify,
}

/// A set of style properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleProperties {
    pub font_name: Option<String>,
    pub font_size: Option<Dimension>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<String>,  // "normal", "italic", "oblique"
    pub text_color: Option<String>,
    pub background_color: Option<String>,
    pub line_height: Option<f64>,
    pub paragraph_indent: Option<Dimension>,
    pub space_before: Option<Dimension>,
    pub space_after: Option<Dimension>,
    pub text_align: Option<TextAlign>,
    pub keep_with_next: Option<bool>,
    pub page_break_before: Option<bool>,
    pub first_line_indent: Option<Dimension>,
    pub margins: Option<(Dimension, Dimension, Dimension, Dimension)>, // top, right, bottom, left
}

/// A named style declaration with optional parent for inheritance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDecl {
    pub name: String,
    pub parent: Option<String>,
    pub properties: StyleProperties,
}

/// Collection of style declarations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleDecls {
    pub styles: Vec<StyleDecl>,
}

impl StyleDecls {
    pub fn find(&self, name: &str) -> Option<&StyleDecl> {
        self.styles.iter().find(|s| s.name == name)
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut StyleDecl> {
        self.styles.iter_mut().find(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_decl_find() {
        let mut decls = StyleDecls::default();
        decls.styles.push(StyleDecl {
            name: "body".to_string(),
            parent: None,
            properties: StyleProperties::default(),
        });
        decls.styles.push(StyleDecl {
            name: "heading".to_string(),
            parent: Some("body".to_string()),
            properties: StyleProperties::default(),
        });

        assert!(decls.find("body").is_some());
        assert!(decls.find("heading").is_some());
        assert!(decls.find("nonexistent").is_none());
        assert_eq!(decls.find("heading").unwrap().parent.as_deref(), Some("body"));
    }

    #[test]
    fn test_style_properties_default() {
        let props = StyleProperties::default();
        assert!(props.font_name.is_none());
        assert!(props.font_size.is_none());
        assert!(props.font_weight.is_none());
        assert!(props.text_color.is_none());
        assert!(props.margins.is_none());
    }
}
