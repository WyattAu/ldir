//! Document metadata for S-IR v2.

use serde::{Deserialize, Serialize};

/// Document direction for bidirectional text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    Auto,
}

/// A physical dimension with units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Dimension {
    Pt(f64),      // points (1/72 inch)
    Mm(f64),      // millimeters
    In(f64),      // inches
    Cm(f64),      // centimeters
    Percent(f64), // percentage of parent
}

impl Dimension {
    pub fn to_points(&self) -> f64 {
        match self {
            Dimension::Pt(v) => *v,
            Dimension::Mm(v) => v * 72.0 / 25.4,
            Dimension::In(v) => v * 72.0,
            Dimension::Cm(v) => v * 72.0 / 2.54,
            Dimension::Percent(_) => 0.0, // context-dependent
        }
    }
}

/// Page geometry specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageGeometry {
    pub width: Dimension,
    pub height: Dimension,
    pub margin_top: Dimension,
    pub margin_bottom: Dimension,
    pub margin_left: Dimension,
    pub margin_right: Dimension,
    pub column_count: u8,
    pub column_gap: Dimension,
}

impl Default for PageGeometry {
    fn default() -> Self {
        Self {
            width: Dimension::In(8.5),
            height: Dimension::In(11.0),
            margin_top: Dimension::In(1.0),
            margin_bottom: Dimension::In(1.0),
            margin_left: Dimension::In(1.0),
            margin_right: Dimension::In(1.0),
            column_count: 1,
            column_gap: Dimension::Pt(24.0),
        }
    }
}

/// Page style for headers, footers, and custom page dimensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageStyle {
    pub page_width: f64,
    pub page_height: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub margin_left: f64,
    pub margin_right: f64,
    pub header_text: Option<String>,
    pub footer_text: Option<String>,
    pub header_page_number: bool,
    pub footer_page_number: bool,
}

impl Default for PageStyle {
    fn default() -> Self {
        Self {
            page_width: 612.0,
            page_height: 792.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
            margin_left: 72.0,
            margin_right: 72.0,
            header_text: None,
            footer_text: None,
            header_page_number: false,
            footer_page_number: false,
        }
    }
}

/// Document-level metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub language: String, // BCP 47
    pub direction: Direction,
    pub document_class: Option<String>, // "article", "book", "report"
    pub page_geometry: Option<PageGeometry>,
    pub page_style: Option<PageStyle>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            subject: None,
            date: None,
            language: "en".to_string(),
            direction: Direction::Auto,
            document_class: None,
            page_geometry: Some(PageGeometry::default()),
            page_style: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_to_points() {
        let eps = 1e-6;
        assert!((Dimension::Pt(72.0).to_points() - 72.0).abs() < eps);
        assert!((Dimension::In(1.0).to_points() - 72.0).abs() < eps);
        assert!((Dimension::Mm(25.4).to_points() - 72.0).abs() < eps);
        assert!((Dimension::Cm(2.54).to_points() - 72.0).abs() < eps);
        assert_eq!(Dimension::Percent(50.0).to_points(), 0.0);
    }

    #[test]
    fn test_page_geometry_default() {
        let pg = PageGeometry::default();
        assert_eq!(pg.width, Dimension::In(8.5));
        assert_eq!(pg.height, Dimension::In(11.0));
        assert_eq!(pg.column_count, 1);
        assert_eq!(pg.column_gap, Dimension::Pt(24.0));
    }

    #[test]
    fn test_metadata_default() {
        let m = DocumentMetadata::default();
        assert!(m.title.is_none());
        assert!(m.author.is_none());
        assert_eq!(m.language, "en");
        assert_eq!(m.direction, Direction::Auto);
        assert!(m.page_geometry.is_some());
        assert!(m.page_style.is_none());
    }

    #[test]
    fn test_page_style_default() {
        let ps = PageStyle::default();
        assert_eq!(ps.page_width, 612.0);
        assert_eq!(ps.page_height, 792.0);
        assert_eq!(ps.margin_top, 72.0);
        assert_eq!(ps.margin_bottom, 72.0);
        assert_eq!(ps.margin_left, 72.0);
        assert_eq!(ps.margin_right, 72.0);
        assert!(ps.header_text.is_none());
        assert!(ps.footer_text.is_none());
        assert!(!ps.header_page_number);
        assert!(!ps.footer_page_number);
    }

    #[test]
    fn test_page_style_custom_dimensions() {
        let ps = PageStyle {
            page_width: 595.28,
            page_height: 841.89,
            margin_top: 50.0,
            margin_bottom: 50.0,
            margin_left: 60.0,
            margin_right: 60.0,
            header_text: Some("My Report".to_string()),
            footer_text: None,
            header_page_number: true,
            footer_page_number: true,
        };
        assert_eq!(ps.page_width, 595.28);
        assert_eq!(ps.page_height, 841.89);
        assert_eq!(ps.header_text.as_deref(), Some("My Report"));
        assert!(ps.header_page_number);
        assert!(ps.footer_page_number);
    }

    #[test]
    fn test_metadata_with_page_style() {
        let mut m = DocumentMetadata::default();
        m.page_style = Some(PageStyle {
            page_width: 500.0,
            page_height: 700.0,
            ..PageStyle::default()
        });
        assert!(m.page_style.is_some());
        assert_eq!(m.page_style.as_ref().unwrap().page_width, 500.0);
    }
}
