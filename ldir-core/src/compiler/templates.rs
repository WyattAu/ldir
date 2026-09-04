//! Page and document template system for headers, footers, and document layouts.
//!
//! Provides:
//! - [`crate::compiler::templates::PageTemplate`] — per-page header/footer template expansion (legacy, retained)
//! - [`crate::compiler::templates::DocumentTemplate`] — full document template with styles, page layout, and preamble
//! - TOML-based template serialization and built-in template presets

#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Legacy page template (backward compatibility)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// Legacy per-page header/footer template (retained for backward compatibility).
pub struct PageTemplate {
    /// Left header template text.
    pub header_left: String,
    /// Center header template text.
    pub header_center: String,
    /// Right header template text.
    pub header_right: String,
    /// Left footer template text.
    pub footer_left: String,
    /// Center footer template text.
    pub footer_center: String,
    /// Right footer template text.
    pub footer_right: String,
    /// Whether to draw a rule below the header.
    pub header_rule: bool,
    /// Whether to draw a rule above the footer.
    pub footer_rule: bool,
}

impl Default for PageTemplate {
    fn default() -> Self {
        Self {
            header_left: String::new(),
            header_center: String::new(),
            header_right: String::new(),
            footer_left: String::new(),
            footer_center: String::new(),
            footer_right: "%page".into(),
            header_rule: false,
            footer_rule: true,
        }
    }
}

#[derive(Debug, Clone)]
/// Values substituted into `%`-placeholders during template expansion.
pub struct TemplateContext {
    /// Current page number (1-indexed).
    pub page: usize,
    /// Total page count.
    pub pages: usize,
    /// Document title.
    pub title: String,
    /// Current chapter title or number.
    pub chapter: String,
    /// Current section number.
    pub section: String,
    /// Formatted document date.
    pub date: String,
    /// Source file name.
    pub file: String,
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self {
            page: 1,
            pages: 1,
            title: String::new(),
            chapter: String::new(),
            section: String::new(),
            date: String::new(),
            file: String::new(),
        }
    }
}

fn expand_template(template: &str, ctx: &TemplateContext) -> String {
    let mut result = template.to_string();
    result = result.replace("%pages", &ctx.pages.to_string());
    result = result.replace("%page", &ctx.page.to_string());
    result = result.replace("%date", &ctx.date);
    result = result.replace("%title", &ctx.title);
    result = result.replace("%chapter", &ctx.chapter);
    result = result.replace("%section", &ctx.section);
    result = result.replace("%file", &ctx.file);
    result
}

impl PageTemplate {
    /// Expands the header templates; returns `(left, center, right)`.
    pub fn expand_header(&self, ctx: &TemplateContext) -> (String, String, String) {
        (
            expand_template(&self.header_left, ctx),
            expand_template(&self.header_center, ctx),
            expand_template(&self.header_right, ctx),
        )
    }

    /// Expands the footer templates; returns `(left, center, right)`.
    pub fn expand_footer(&self, ctx: &TemplateContext) -> (String, String, String) {
        (
            expand_template(&self.footer_left, ctx),
            expand_template(&self.footer_center, ctx),
            expand_template(&self.footer_right, ctx),
        )
    }
}

// ---------------------------------------------------------------------------
// Document template system
// ---------------------------------------------------------------------------

/// A reusable document template that defines styles, page layout, and preamble.
#[derive(Debug, Clone)]
pub struct DocumentTemplate {
    /// Template name.
    pub name: String,
    /// Default page size.
    pub page_size: PageSize,
    /// Page margins.
    pub margins: Margins,
    /// Body font family name.
    pub font_family: String,
    /// Body font size in points.
    pub font_size: i32,
    /// Line spacing multiplier.
    pub line_spacing: f64,
    /// First-line paragraph indent in points, if any.
    pub paragraph_indent: Option<i32>,
    /// Space between paragraphs in points.
    pub paragraph_spacing: i32,
    /// Style overrides per heading level (1-6).
    pub heading_styles: HashMap<u8, HeadingStyle>,
    /// Header template, if the page has one.
    pub header: Option<HeaderFooterTemplate>,
    /// Footer template, if the page has one.
    pub footer: Option<HeaderFooterTemplate>,
    /// First-page overrides, if any.
    pub first_page: Option<FirstPageTemplate>,
}

#[derive(Debug, Clone)]
/// Typographic style for one heading level.
pub struct HeadingStyle {
    /// Heading font size in points.
    pub font_size: i32,
    /// Whether the heading is bold.
    pub bold: bool,
    /// Whether the heading is italic.
    pub italic: bool,
    /// Space above the heading in points.
    pub spacing_before: i32,
    /// Space below the heading in points.
    pub spacing_after: i32,
}

#[derive(Debug, Clone)]
/// Page margins in points.
pub struct Margins {
    /// Top margin.
    pub top: i32,
    /// Bottom margin.
    pub bottom: i32,
    /// Left margin.
    pub left: i32,
    /// Right margin.
    pub right: i32,
}

#[derive(Debug, Clone)]
/// Named page sizes plus a custom option.
pub enum PageSize {
    /// ISO A4 (595 x 842 pt).
    A4,
    /// US Letter (612 x 792 pt).
    Letter,
    /// US Legal (612 x 1008 pt).
    Legal,
    /// Caller-specified size in points.
    Custom {
        /// Page width in points.
        width: i32,
        /// Page height in points.
        height: i32,
    },
}

#[derive(Debug, Clone)]
/// Header/footer template with left/center/right slots and an optional rule.
pub struct HeaderFooterTemplate {
    /// Left slot template text.
    pub left: String,
    /// Center slot template text.
    pub center: String,
    /// Right slot template text.
    pub right: String,
    /// Whether to draw the rule.
    pub rule: bool,
}

#[derive(Debug, Clone)]
/// Overrides applied to the first page of the document.
pub struct FirstPageTemplate {
    /// Whether the first page uses a different header.
    pub different_header: bool,
    /// Whether the first page uses a different footer.
    pub different_footer: bool,
    /// Whether the header is suppressed on the first page.
    pub suppress_header: bool,
    /// Whether the page number is suppressed on the first page.
    pub suppress_page_number: bool,
}

impl PageSize {
    /// Returns `(width, height)` in points.
    pub fn dimensions(&self) -> (i32, i32) {
        match self {
            PageSize::A4 => (595, 842),
            PageSize::Letter => (612, 792),
            PageSize::Legal => (612, 1008),
            PageSize::Custom { width, height } => (*width, *height),
        }
    }

    /// Looks up a page size by case-insensitive name (`a4`, `letter`, `legal`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "a4" => Some(PageSize::A4),
            "letter" => Some(PageSize::Letter),
            "legal" => Some(PageSize::Legal),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Built-in templates
// ---------------------------------------------------------------------------

impl DocumentTemplate {
    /// Returns the built-in `academic` template (serif body, numbered headings, tight line spacing).
    pub fn academic() -> Self {
        let mut heading_styles = HashMap::new();
        heading_styles.insert(
            1,
            HeadingStyle {
                font_size: 24,
                bold: true,
                italic: false,
                spacing_before: 24,
                spacing_after: 12,
            },
        );
        heading_styles.insert(
            2,
            HeadingStyle {
                font_size: 18,
                bold: true,
                italic: false,
                spacing_before: 18,
                spacing_after: 8,
            },
        );
        heading_styles.insert(
            3,
            HeadingStyle {
                font_size: 14,
                bold: true,
                italic: false,
                spacing_before: 14,
                spacing_after: 6,
            },
        );
        heading_styles.insert(
            4,
            HeadingStyle {
                font_size: 12,
                bold: true,
                italic: true,
                spacing_before: 10,
                spacing_after: 4,
            },
        );

        Self {
            name: "academic".to_string(),
            page_size: PageSize::A4,
            margins: Margins {
                top: 72,
                bottom: 72,
                left: 72,
                right: 72,
            },
            font_family: "DejaVu Serif".to_string(),
            font_size: 11,
            line_spacing: 1.15,
            paragraph_indent: Some(20),
            paragraph_spacing: 6,
            heading_styles,
            header: Some(HeaderFooterTemplate {
                left: String::new(),
                center: String::new(),
                right: "%title".to_string(),
                rule: false,
            }),
            footer: Some(HeaderFooterTemplate {
                left: String::new(),
                center: "%page of %pages".to_string(),
                right: String::new(),
                rule: true,
            }),
            first_page: Some(FirstPageTemplate {
                different_header: true,
                different_footer: true,
                suppress_header: true,
                suppress_page_number: true,
            }),
        }
    }

    /// Returns the built-in `report` template (sans headings, generous spacing).
    pub fn report() -> Self {
        let mut heading_styles = HashMap::new();
        heading_styles.insert(
            1,
            HeadingStyle {
                font_size: 26,
                bold: true,
                italic: false,
                spacing_before: 30,
                spacing_after: 16,
            },
        );
        heading_styles.insert(
            2,
            HeadingStyle {
                font_size: 20,
                bold: true,
                italic: false,
                spacing_before: 20,
                spacing_after: 10,
            },
        );
        heading_styles.insert(
            3,
            HeadingStyle {
                font_size: 14,
                bold: false,
                italic: false,
                spacing_before: 16,
                spacing_after: 8,
            },
        );
        heading_styles.insert(
            4,
            HeadingStyle {
                font_size: 12,
                bold: true,
                italic: false,
                spacing_before: 12,
                spacing_after: 6,
            },
        );

        Self {
            name: "report".to_string(),
            page_size: PageSize::Letter,
            margins: Margins {
                top: 72,
                bottom: 72,
                left: 72,
                right: 72,
            },
            font_family: "DejaVu Sans".to_string(),
            font_size: 12,
            line_spacing: 1.25,
            paragraph_indent: None,
            paragraph_spacing: 8,
            heading_styles,
            header: Some(HeaderFooterTemplate {
                left: "%title".to_string(),
                center: String::new(),
                right: "%date".to_string(),
                rule: true,
            }),
            footer: Some(HeaderFooterTemplate {
                left: String::new(),
                center: String::new(),
                right: "Page %page".to_string(),
                rule: true,
            }),
            first_page: Some(FirstPageTemplate {
                different_header: true,
                different_footer: false,
                suppress_header: true,
                suppress_page_number: false,
            }),
        }
    }

    /// Returns the built-in `book` template (larger margins, running headers).
    pub fn book() -> Self {
        let mut heading_styles = HashMap::new();
        heading_styles.insert(
            1,
            HeadingStyle {
                font_size: 28,
                bold: true,
                italic: false,
                spacing_before: 0,
                spacing_after: 20,
            },
        );
        heading_styles.insert(
            2,
            HeadingStyle {
                font_size: 22,
                bold: true,
                italic: false,
                spacing_before: 24,
                spacing_after: 12,
            },
        );
        heading_styles.insert(
            3,
            HeadingStyle {
                font_size: 16,
                bold: false,
                italic: false,
                spacing_before: 18,
                spacing_after: 8,
            },
        );
        heading_styles.insert(
            4,
            HeadingStyle {
                font_size: 13,
                bold: true,
                italic: false,
                spacing_before: 14,
                spacing_after: 6,
            },
        );

        Self {
            name: "book".to_string(),
            page_size: PageSize::A4,
            margins: Margins {
                top: 60,
                bottom: 60,
                left: 72,
                right: 72,
            },
            font_family: "DejaVu Serif".to_string(),
            font_size: 11,
            line_spacing: 1.4,
            paragraph_indent: Some(24),
            paragraph_spacing: 4,
            heading_styles,
            header: Some(HeaderFooterTemplate {
                left: "%chapter".to_string(),
                center: String::new(),
                right: "%page".to_string(),
                rule: true,
            }),
            footer: Some(HeaderFooterTemplate {
                left: String::new(),
                center: String::new(),
                right: String::new(),
                rule: false,
            }),
            first_page: Some(FirstPageTemplate {
                different_header: true,
                different_footer: false,
                suppress_header: true,
                suppress_page_number: true,
            }),
        }
    }

    /// Returns the built-in `letter` template (business-letter defaults).
    pub fn letter() -> Self {
        let mut heading_styles = HashMap::new();
        heading_styles.insert(
            1,
            HeadingStyle {
                font_size: 16,
                bold: true,
                italic: false,
                spacing_before: 12,
                spacing_after: 8,
            },
        );
        heading_styles.insert(
            2,
            HeadingStyle {
                font_size: 13,
                bold: true,
                italic: false,
                spacing_before: 10,
                spacing_after: 6,
            },
        );
        heading_styles.insert(
            3,
            HeadingStyle {
                font_size: 12,
                bold: false,
                italic: true,
                spacing_before: 8,
                spacing_after: 4,
            },
        );
        heading_styles.insert(
            4,
            HeadingStyle {
                font_size: 11,
                bold: true,
                italic: false,
                spacing_before: 6,
                spacing_after: 3,
            },
        );

        Self {
            name: "letter".to_string(),
            page_size: PageSize::Letter,
            margins: Margins {
                top: 72,
                bottom: 72,
                left: 96,
                right: 72,
            },
            font_family: "DejaVu Sans".to_string(),
            font_size: 12,
            line_spacing: 1.15,
            paragraph_indent: None,
            paragraph_spacing: 6,
            heading_styles,
            header: None,
            footer: Some(HeaderFooterTemplate {
                left: String::new(),
                center: String::new(),
                right: "Page %page".to_string(),
                rule: false,
            }),
            first_page: Some(FirstPageTemplate {
                different_header: false,
                different_footer: true,
                suppress_header: false,
                suppress_page_number: true,
            }),
        }
    }

    /// Returns the built-in `minimal` template (plain defaults, no headers/footers).
    pub fn minimal() -> Self {
        let mut heading_styles = HashMap::new();
        heading_styles.insert(
            1,
            HeadingStyle {
                font_size: 24,
                bold: true,
                italic: false,
                spacing_before: 18,
                spacing_after: 10,
            },
        );
        heading_styles.insert(
            2,
            HeadingStyle {
                font_size: 18,
                bold: true,
                italic: false,
                spacing_before: 14,
                spacing_after: 8,
            },
        );
        heading_styles.insert(
            3,
            HeadingStyle {
                font_size: 14,
                bold: true,
                italic: false,
                spacing_before: 10,
                spacing_after: 6,
            },
        );
        heading_styles.insert(
            4,
            HeadingStyle {
                font_size: 12,
                bold: true,
                italic: false,
                spacing_before: 8,
                spacing_after: 4,
            },
        );

        Self {
            name: "minimal".to_string(),
            page_size: PageSize::Letter,
            margins: Margins {
                top: 72,
                bottom: 72,
                left: 72,
                right: 72,
            },
            font_family: "DejaVu Sans".to_string(),
            font_size: 12,
            line_spacing: 1.2,
            paragraph_indent: None,
            paragraph_spacing: 8,
            heading_styles,
            header: None,
            footer: Some(HeaderFooterTemplate {
                left: String::new(),
                center: String::new(),
                right: "%page".to_string(),
                rule: true,
            }),
            first_page: None,
        }
    }

    /// Looks up a built-in template by name (`academic`, `report`, `book`, `letter`, `minimal`).
    pub fn builtin(name: &str) -> Option<Self> {
        match name {
            "academic" => Some(Self::academic()),
            "report" => Some(Self::report()),
            "book" => Some(Self::book()),
            "letter" => Some(Self::letter()),
            "minimal" => Some(Self::minimal()),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// TOML serialization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, thiserror::Error)]
/// Errors from parsing or applying document templates.
pub enum TemplateError {
    #[error("TOML parse error: {0}")]
    /// The TOML could not be deserialized; contains the parser error.
    TomlParse(#[from] toml::de::Error),
    #[error("unknown page size: {0}")]
    /// The template names a page size that is not built in.
    UnknownPageSize(String),
    #[error("invalid heading level: {0}")]
    /// A heading level outside 1-6 was configured.
    InvalidHeadingLevel(u8),
}

#[derive(Deserialize)]
struct TomlTemplate {
    name: Option<String>,
    page_size: Option<String>,
    page_width: Option<i32>,
    page_height: Option<i32>,
    margins: Option<TomlMargins>,
    font_family: Option<String>,
    font_size: Option<i32>,
    line_spacing: Option<f64>,
    paragraph_indent: Option<i32>,
    paragraph_spacing: Option<i32>,
    heading_styles: Option<HashMap<String, TomlHeadingStyle>>,
    header: Option<TomlHeaderFooter>,
    footer: Option<TomlHeaderFooter>,
    first_page: Option<TomlFirstPage>,
}

#[derive(Deserialize)]
struct TomlMargins {
    top: Option<i32>,
    bottom: Option<i32>,
    left: Option<i32>,
    right: Option<i32>,
}

#[derive(Deserialize)]
struct TomlHeadingStyle {
    font_size: Option<i32>,
    bold: Option<bool>,
    italic: Option<bool>,
    spacing_before: Option<i32>,
    spacing_after: Option<i32>,
}

#[derive(Deserialize)]
struct TomlHeaderFooter {
    left: Option<String>,
    center: Option<String>,
    right: Option<String>,
    rule: Option<bool>,
}

#[derive(Deserialize)]
struct TomlFirstPage {
    different_header: Option<bool>,
    different_footer: Option<bool>,
    suppress_header: Option<bool>,
    suppress_page_number: Option<bool>,
}

/// Parses a TOML document template into a [`DocumentTemplate`].
pub fn parse_template(toml: &str) -> Result<DocumentTemplate, TemplateError> {
    let raw: TomlTemplate = toml::from_str(toml)?;
    let defaults = DocumentTemplate::minimal();

    let page_size = if let (Some(w), Some(h)) = (raw.page_width, raw.page_height) {
        PageSize::Custom {
            width: w,
            height: h,
        }
    } else if let Some(ps) = &raw.page_size {
        PageSize::from_name(ps).ok_or_else(|| TemplateError::UnknownPageSize(ps.clone()))?
    } else {
        defaults.page_size.clone()
    };

    let margins = Margins {
        top: raw
            .margins
            .as_ref()
            .and_then(|m| m.top)
            .unwrap_or(defaults.margins.top),
        bottom: raw
            .margins
            .as_ref()
            .and_then(|m| m.bottom)
            .unwrap_or(defaults.margins.bottom),
        left: raw
            .margins
            .as_ref()
            .and_then(|m| m.left)
            .unwrap_or(defaults.margins.left),
        right: raw
            .margins
            .as_ref()
            .and_then(|m| m.right)
            .unwrap_or(defaults.margins.right),
    };

    let mut heading_styles = HashMap::new();
    if let Some(ref hs) = raw.heading_styles {
        for (key, val) in hs {
            let level: u8 = key
                .parse()
                .map_err(|_| TemplateError::InvalidHeadingLevel(0))?;
            if !(1..=6).contains(&level) {
                return Err(TemplateError::InvalidHeadingLevel(level));
            }
            let default_hs = defaults
                .heading_styles
                .get(&level)
                .cloned()
                .unwrap_or(HeadingStyle {
                    font_size: 12,
                    bold: false,
                    italic: false,
                    spacing_before: 8,
                    spacing_after: 4,
                });
            heading_styles.insert(
                level,
                HeadingStyle {
                    font_size: val.font_size.unwrap_or(default_hs.font_size),
                    bold: val.bold.unwrap_or(default_hs.bold),
                    italic: val.italic.unwrap_or(default_hs.italic),
                    spacing_before: val.spacing_before.unwrap_or(default_hs.spacing_before),
                    spacing_after: val.spacing_after.unwrap_or(default_hs.spacing_after),
                },
            );
        }
    } else {
        heading_styles = defaults.heading_styles;
    }

    let header = raw.header.map(|h| HeaderFooterTemplate {
        left: h.left.unwrap_or_default(),
        center: h.center.unwrap_or_default(),
        right: h.right.unwrap_or_default(),
        rule: h.rule.unwrap_or(false),
    });

    let footer = raw.footer.map(|f| HeaderFooterTemplate {
        left: f.left.unwrap_or_default(),
        center: f.center.unwrap_or_default(),
        right: f.right.unwrap_or_default(),
        rule: f.rule.unwrap_or(true),
    });

    let first_page = raw.first_page.map(|fp| FirstPageTemplate {
        different_header: fp.different_header.unwrap_or(false),
        different_footer: fp.different_footer.unwrap_or(false),
        suppress_header: fp.suppress_header.unwrap_or(false),
        suppress_page_number: fp.suppress_page_number.unwrap_or(false),
    });

    Ok(DocumentTemplate {
        name: raw.name.unwrap_or_else(|| defaults.name.clone()),
        page_size,
        margins,
        font_family: raw
            .font_family
            .unwrap_or_else(|| defaults.font_family.clone()),
        font_size: raw.font_size.unwrap_or(defaults.font_size),
        line_spacing: raw.line_spacing.unwrap_or(defaults.line_spacing),
        paragraph_indent: raw.paragraph_indent.or(defaults.paragraph_indent),
        paragraph_spacing: raw.paragraph_spacing.unwrap_or(defaults.paragraph_spacing),
        heading_styles,
        header,
        footer,
        first_page,
    })
}

// ---------------------------------------------------------------------------
// Template -> CompileContext conversion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
/// A flat snapshot of a [`DocumentTemplate`] used to seed the compiler context.
pub struct CompileContextPartial {
    /// Page width in points.
    pub page_width_pt: i32,
    /// Page height in points.
    pub page_height_pt: i32,
    /// Top margin in points.
    pub margin_top_pt: i32,
    /// Bottom margin in points.
    pub margin_bottom_pt: i32,
    /// Left margin in points.
    pub margin_left_pt: i32,
    /// Right margin in points.
    pub margin_right_pt: i32,
    /// Body font family name.
    pub font_family: String,
    /// Body font size in points.
    pub font_size_pt: i32,
    /// Line spacing multiplier.
    pub line_spacing: f64,
    /// First-line paragraph indent in points, if any.
    pub paragraph_indent_pt: Option<i32>,
    /// Space between paragraphs in points.
    pub paragraph_spacing_pt: i32,
    /// Style overrides per heading level (1-6).
    pub heading_styles: HashMap<u8, HeadingStyle>,
    /// Legacy page template, if present.
    pub page_template: Option<PageTemplate>,
    /// First-page overrides, if any.
    pub first_page: Option<FirstPageTemplate>,
}

/// Converts a document template into the partial context used to seed compilation.
pub fn template_to_context(template: &DocumentTemplate) -> CompileContextPartial {
    let (pw, ph) = template.page_size.dimensions();

    let page_template = if template.header.is_some() || template.footer.is_some() {
        Some(PageTemplate {
            header_left: template
                .header
                .as_ref()
                .map(|h| h.left.clone())
                .unwrap_or_default(),
            header_center: template
                .header
                .as_ref()
                .map(|h| h.center.clone())
                .unwrap_or_default(),
            header_right: template
                .header
                .as_ref()
                .map(|h| h.right.clone())
                .unwrap_or_default(),
            footer_left: template
                .footer
                .as_ref()
                .map(|f| f.left.clone())
                .unwrap_or_default(),
            footer_center: template
                .footer
                .as_ref()
                .map(|f| f.center.clone())
                .unwrap_or_default(),
            footer_right: template
                .footer
                .as_ref()
                .map(|f| f.right.clone())
                .unwrap_or_else(|| "%page".to_string()),
            header_rule: template.header.as_ref().map(|h| h.rule).unwrap_or(false),
            footer_rule: template.footer.as_ref().map(|f| f.rule).unwrap_or(true),
        })
    } else {
        None
    };

    CompileContextPartial {
        page_width_pt: pw,
        page_height_pt: ph,
        margin_top_pt: template.margins.top,
        margin_bottom_pt: template.margins.bottom,
        margin_left_pt: template.margins.left,
        margin_right_pt: template.margins.right,
        font_family: template.font_family.clone(),
        font_size_pt: template.font_size,
        line_spacing: template.line_spacing,
        paragraph_indent_pt: template.paragraph_indent,
        paragraph_spacing_pt: template.paragraph_spacing,
        heading_styles: template.heading_styles.clone(),
        page_template,
        first_page: template.first_page.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_template() {
        let tmpl = PageTemplate::default();
        assert!(tmpl.header_left.is_empty());
        assert!(tmpl.header_center.is_empty());
        assert!(tmpl.header_right.is_empty());
        assert!(tmpl.footer_left.is_empty());
        assert!(tmpl.footer_center.is_empty());
        assert_eq!(tmpl.footer_right, "%page");
        assert!(!tmpl.header_rule);
        assert!(tmpl.footer_rule);
    }

    #[test]
    fn test_default_context() {
        let ctx = TemplateContext::default();
        assert_eq!(ctx.page, 1);
        assert_eq!(ctx.pages, 1);
        assert!(ctx.title.is_empty());
    }

    #[test]
    fn test_expand_footer_page_number() {
        let tmpl = PageTemplate::default();
        let ctx = TemplateContext {
            page: 5,
            pages: 10,
            ..Default::default()
        };
        let (left, center, right) = tmpl.expand_footer(&ctx);
        assert_eq!(left, "");
        assert_eq!(center, "");
        assert_eq!(right, "5");
    }

    #[test]
    fn test_expand_footer_pages_total() {
        let tmpl = PageTemplate {
            footer_right: "Page %page of %pages".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            page: 3,
            pages: 20,
            ..Default::default()
        };
        let (_, _, right) = tmpl.expand_footer(&ctx);
        assert_eq!(right, "Page 3 of 20");
    }

    #[test]
    fn test_expand_header_title() {
        let tmpl = PageTemplate {
            header_center: "%title".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            title: "My Document".into(),
            ..Default::default()
        };
        let (_, center, _) = tmpl.expand_header(&ctx);
        assert_eq!(center, "My Document");
    }

    #[test]
    fn test_expand_header_chapter() {
        let tmpl = PageTemplate {
            header_right: "%chapter".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            chapter: "Introduction".into(),
            ..Default::default()
        };
        let (_, _, right) = tmpl.expand_header(&ctx);
        assert_eq!(right, "Introduction");
    }

    #[test]
    fn test_expand_footer_section() {
        let tmpl = PageTemplate {
            footer_left: "%section".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            section: "2.1 Methods".into(),
            ..Default::default()
        };
        let (left, _, _) = tmpl.expand_footer(&ctx);
        assert_eq!(left, "2.1 Methods");
    }

    #[test]
    fn test_expand_footer_date() {
        let tmpl = PageTemplate {
            footer_center: "%date".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            date: "2026-05-01".into(),
            ..Default::default()
        };
        let (_, center, _) = tmpl.expand_footer(&ctx);
        assert_eq!(center, "2026-05-01");
    }

    #[test]
    fn test_expand_footer_file() {
        let tmpl = PageTemplate {
            footer_left: "%file".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            file: "input.md".into(),
            ..Default::default()
        };
        let (left, _, _) = tmpl.expand_footer(&ctx);
        assert_eq!(left, "input.md");
    }

    #[test]
    fn test_expand_multiple_variables() {
        let tmpl = PageTemplate {
            header_right: "%title - %chapter (%page/%pages)".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            page: 7,
            pages: 42,
            title: "Thesis".into(),
            chapter: "Conclusion".into(),
            ..Default::default()
        };
        let (_, _, right) = tmpl.expand_header(&ctx);
        assert_eq!(right, "Thesis - Conclusion (7/42)");
    }

    #[test]
    fn test_expand_no_variables() {
        let tmpl = PageTemplate {
            footer_right: "Confidential".into(),
            ..Default::default()
        };
        let ctx = TemplateContext::default();
        let (_, _, right) = tmpl.expand_footer(&ctx);
        assert_eq!(right, "Confidential");
    }

    #[test]
    fn test_expand_empty_template() {
        let tmpl = PageTemplate::default();
        let ctx = TemplateContext::default();
        let (left, center, right) = tmpl.expand_header(&ctx);
        assert!(left.is_empty());
        assert!(center.is_empty());
        assert!(right.is_empty());
    }

    #[test]
    fn test_expand_all_fields() {
        let tmpl = PageTemplate {
            header_left: "%file".into(),
            header_center: "%title".into(),
            header_right: "%date".into(),
            footer_left: "%chapter".into(),
            footer_center: "%section".into(),
            footer_right: "%page of %pages".into(),
            header_rule: true,
            footer_rule: true,
        };
        let ctx = TemplateContext {
            page: 1,
            pages: 5,
            title: "Doc".into(),
            chapter: "Ch1".into(),
            section: "Sec1".into(),
            date: "Today".into(),
            file: "doc.md".into(),
        };
        let (hl, hc, hr) = tmpl.expand_header(&ctx);
        assert_eq!(hl, "doc.md");
        assert_eq!(hc, "Doc");
        assert_eq!(hr, "Today");
        let (fl, fc, fr) = tmpl.expand_footer(&ctx);
        assert_eq!(fl, "Ch1");
        assert_eq!(fc, "Sec1");
        assert_eq!(fr, "1 of 5");
        assert!(tmpl.header_rule);
        assert!(tmpl.footer_rule);
    }

    #[test]
    fn test_expand_repeated_variable() {
        let tmpl = PageTemplate {
            footer_right: "%page-%page-%page".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            page: 3,
            ..Default::default()
        };
        let (_, _, right) = tmpl.expand_footer(&ctx);
        assert_eq!(right, "3-3-3");
    }

    #[test]
    fn test_expand_unknown_variable_passthrough() {
        let tmpl = PageTemplate {
            footer_right: "%unknown %page".into(),
            ..Default::default()
        };
        let ctx = TemplateContext {
            page: 1,
            ..Default::default()
        };
        let (_, _, right) = tmpl.expand_footer(&ctx);
        assert_eq!(right, "%unknown 1");
    }

    #[test]
    fn test_template_clone() {
        let tmpl = PageTemplate {
            footer_right: "test".into(),
            ..Default::default()
        };
        let cloned = tmpl.clone();
        assert_eq!(cloned.footer_right, "test");
    }

    // -----------------------------------------------------------------------
    // DocumentTemplate tests
    // -----------------------------------------------------------------------

    #[test]
    fn academic_template() {
        let t = DocumentTemplate::academic();
        assert_eq!(t.name, "academic");
        assert_eq!(t.font_family, "DejaVu Serif");
        assert_eq!(t.font_size, 11);
        assert_eq!(t.line_spacing, 1.15);
        assert_eq!(t.paragraph_indent, Some(20));
        assert_eq!(t.paragraph_spacing, 6);
        assert_eq!(t.margins.top, 72);
        assert_eq!(t.margins.bottom, 72);
        assert_eq!(t.margins.left, 72);
        assert_eq!(t.margins.right, 72);
        let h1 = t.heading_styles.get(&1).expect("h1");
        assert_eq!(h1.font_size, 24);
        assert!(h1.bold);
        assert!(!h1.italic);
    }

    #[test]
    fn report_template() {
        let t = DocumentTemplate::report();
        assert_eq!(t.name, "report");
        assert_eq!(t.page_size.dimensions(), (612, 792));
        assert_eq!(t.font_family, "DejaVu Sans");
        assert_eq!(t.font_size, 12);
        assert_eq!(t.line_spacing, 1.25);
        assert_eq!(t.paragraph_indent, None);
        assert_eq!(t.paragraph_spacing, 8);
        let h2 = t.heading_styles.get(&2).expect("h2");
        assert_eq!(h2.font_size, 20);
        assert!(h2.bold);
    }

    #[test]
    fn book_template() {
        let t = DocumentTemplate::book();
        assert_eq!(t.name, "book");
        assert_eq!(t.page_size.dimensions(), (595, 842));
        assert_eq!(t.font_family, "DejaVu Serif");
        assert_eq!(t.font_size, 11);
        assert_eq!(t.line_spacing, 1.4);
        assert_eq!(t.paragraph_indent, Some(24));
        assert_eq!(t.paragraph_spacing, 4);
        let h1 = t.heading_styles.get(&1).expect("h1");
        assert_eq!(h1.font_size, 28);
        assert_eq!(h1.spacing_before, 0);
    }

    #[test]
    fn letter_template() {
        let t = DocumentTemplate::letter();
        assert_eq!(t.name, "letter");
        assert_eq!(t.page_size.dimensions(), (612, 792));
        assert_eq!(t.margins.left, 96);
        assert_eq!(t.margins.right, 72);
        assert_eq!(t.font_family, "DejaVu Sans");
        assert_eq!(t.font_size, 12);
        assert_eq!(t.line_spacing, 1.15);
        assert!(t.header.is_none());
    }

    #[test]
    fn minimal_template() {
        let t = DocumentTemplate::minimal();
        assert_eq!(t.name, "minimal");
        assert_eq!(t.page_size.dimensions(), (612, 792));
        assert_eq!(t.font_family, "DejaVu Sans");
        assert_eq!(t.font_size, 12);
        assert_eq!(t.line_spacing, 1.2);
        assert_eq!(t.paragraph_indent, None);
        assert!(t.first_page.is_none());
    }

    #[test]
    fn parse_toml_template() {
        let toml_str = r#"
name = "my-template"
page_size = "a4"
font_family = "DejaVu Serif"
font_size = 11
line_spacing = 1.15
paragraph_indent = 20
paragraph_spacing = 6

[margins]
top = 72
bottom = 72
left = 72
right = 72

[heading_styles.1]
font_size = 24
bold = true
spacing_before = 24
spacing_after = 12

[heading_styles.2]
font_size = 18
bold = true
spacing_before = 18
spacing_after = 8
"#;
        let t = parse_template(toml_str).expect("parse ok");
        assert_eq!(t.name, "my-template");
        assert_eq!(t.page_size.dimensions(), (595, 842));
        assert_eq!(t.font_family, "DejaVu Serif");
        assert_eq!(t.font_size, 11);
        assert_eq!(t.paragraph_indent, Some(20));
        let h1 = t.heading_styles.get(&1).expect("h1");
        assert_eq!(h1.font_size, 24);
        assert!(h1.bold);
    }

    #[test]
    fn template_to_context_conversion() {
        let t = DocumentTemplate::academic();
        let partial = template_to_context(&t);
        assert_eq!(partial.page_width_pt, 595);
        assert_eq!(partial.page_height_pt, 842);
        assert_eq!(partial.margin_top_pt, 72);
        assert_eq!(partial.margin_left_pt, 72);
        assert_eq!(partial.font_family, "DejaVu Serif");
        assert_eq!(partial.font_size_pt, 11);
        assert_eq!(partial.line_spacing, 1.15);
        assert!(partial.page_template.is_some());
    }

    #[test]
    fn custom_page_size() {
        let toml_str = r#"
name = "custom"
page_width = 400
page_height = 600
"#;
        let t = parse_template(toml_str).expect("parse ok");
        assert_eq!(t.page_size.dimensions(), (400, 600));
    }

    #[test]
    fn heading_styles_all_levels() {
        let t = DocumentTemplate::minimal();
        assert!(t.heading_styles.contains_key(&1));
        assert!(t.heading_styles.contains_key(&2));
        assert!(t.heading_styles.contains_key(&3));
        assert!(t.heading_styles.contains_key(&4));
        let h4 = t.heading_styles.get(&4).expect("h4");
        assert_eq!(h4.font_size, 12);
        assert!(h4.bold);
    }

    #[test]
    fn first_page_template() {
        let t = DocumentTemplate::academic();
        let fp = t.first_page.as_ref().expect("first_page");
        assert!(fp.suppress_header);
        assert!(fp.suppress_page_number);
        assert!(fp.different_header);
        assert!(fp.different_footer);
    }
}
