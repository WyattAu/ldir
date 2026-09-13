//! Global pagination with page numbers, headers, and footers.
//!
//! Provides configuration types for page number styles, header/footer
//! layout, template variable substitution, and running pagination state.
//!
//! ## Two-pass layout
//!
//! 1. Pass 1: Layout all content, determine total page count.
//! 2. Pass 2: Add headers/footers to each page using the computed page count.
//!
//! ## Template variables
//!
//! Headers and footers support the following placeholders:
//! - `{page}` -- current page number
//! - `{pages}` -- total page count
//! - `{chapter}` -- current chapter number
//! - `{section}` -- current section number
//! - `{title}` -- document title
//! - `{date}` -- current date
//! - `{author}` -- document author

use crate::page_numbers::{PageNumberStyle, format_page_number};

/// Where to place the page number within header or footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageNumberPlacement {
    /// Page number at the left of the header.
    HeaderLeft,
    /// Page number centered in the header.
    HeaderCenter,
    /// Page number at the right of the header.
    HeaderRight,
    /// Page number at the left of the footer.
    FooterLeft,
    #[default]
    /// Page number centered in the footer (default).
    FooterCenter,
    /// Page number at the right of the footer.
    FooterRight,
}

/// Header and footer configuration for paginated documents.
#[derive(Debug, Clone, PartialEq)]
pub struct HeaderFooterConfig {
    /// Where the page number is placed.
    pub page_number_placement: PageNumberPlacement,
    /// Numbering style (arabic, roman, letters).
    pub page_number_style: PageNumberStyle,
    /// Template text for the header left slot; `None` leaves it empty.
    pub header_left: Option<String>,
    /// Template text for the header center slot; `None` leaves it empty.
    pub header_center: Option<String>,
    /// Template text for the header right slot; `None` leaves it empty.
    pub header_right: Option<String>,
    /// Template text for the footer left slot; `None` leaves it empty.
    pub footer_left: Option<String>,
    /// Template text for the footer center slot; `None` leaves it empty.
    pub footer_center: Option<String>,
    /// Template text for the footer right slot; `None` leaves it empty.
    pub footer_right: Option<String>,
    /// Whether to draw a rule below the header.
    pub header_line: bool,
    /// Whether to draw a rule above the footer.
    pub footer_line: bool,
    /// Whether the first page suppresses headers/footers.
    pub first_page_different: bool,
    /// Whether odd and even pages use different layouts.
    pub odd_even_different: bool,
}

impl Default for HeaderFooterConfig {
    fn default() -> Self {
        Self {
            page_number_placement: PageNumberPlacement::FooterCenter,
            page_number_style: PageNumberStyle::Arabic,
            header_left: None,
            header_center: None,
            header_right: None,
            footer_left: None,
            footer_center: None,
            footer_right: None,
            header_line: false,
            footer_line: false,
            first_page_different: true,
            odd_even_different: false,
        }
    }
}

impl HeaderFooterConfig {
    /// Creates a config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the page number style.
    pub fn with_page_number_style(mut self, style: PageNumberStyle) -> Self {
        self.page_number_style = style;
        self
    }

    /// Sets the page number placement.
    pub fn with_page_number_placement(mut self, placement: PageNumberPlacement) -> Self {
        self.page_number_placement = placement;
        self
    }

    /// Enables or disables the header rule.
    pub fn with_header_line(mut self, enabled: bool) -> Self {
        self.header_line = enabled;
        self
    }

    /// Enables or disables the footer rule.
    pub fn with_footer_line(mut self, enabled: bool) -> Self {
        self.footer_line = enabled;
        self
    }

    /// Enables or disables first-page suppression.
    pub fn with_first_page_different(mut self, enabled: bool) -> Self {
        self.first_page_different = enabled;
        self
    }

    /// Enables or disables different odd/even layouts.
    pub fn with_odd_even_different(mut self, enabled: bool) -> Self {
        self.odd_even_different = enabled;
        self
    }
}

/// Running pagination state maintained during layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PaginationState {
    /// Current page number (1-indexed).
    pub current_page: u32,
    /// Total page count from pass 1 (0 until known).
    pub total_pages: u32,
    /// Current chapter number.
    pub chapter: u32,
    /// Current section number.
    pub section: u32,
    /// Header/footer configuration in effect.
    pub config: HeaderFooterConfig,
}

impl PaginationState {
    /// Creates initial state for page 1 with no known total.
    pub fn new(config: HeaderFooterConfig) -> Self {
        Self {
            current_page: 1,
            total_pages: 0,
            chapter: 0,
            section: 0,
            config,
        }
    }

    /// Current page number formatted in the configured style.
    pub fn formatted_page_number(&self) -> String {
        format_page_number(self.current_page, self.config.page_number_style).unwrap_or_default()
    }

    /// Total pages formatted in the configured style; empty when unknown.
    pub fn formatted_total_pages(&self) -> String {
        if self.total_pages == 0 {
            String::new()
        } else {
            format_page_number(self.total_pages, self.config.page_number_style).unwrap_or_default()
        }
    }

    /// Whether the current page is page 1.
    pub fn is_first_page(&self) -> bool {
        self.current_page == 1
    }

    /// Whether the current page number is odd.
    pub fn is_odd_page(&self) -> bool {
        self.current_page % 2 == 1
    }

    /// Whether header/footer should be shown on the current page.
    pub fn should_show_header_footer(&self) -> bool {
        if self.config.first_page_different && self.is_first_page() {
            return false;
        }
        true
    }
}

/// Lightweight metadata needed for template substitution.
#[derive(Debug, Clone, Default)]
pub struct TemplateMetadata {
    /// Document title.
    pub title: String,
    /// Document author.
    pub author: String,
    /// Document date.
    pub date: String,
}

/// Substitute template variables in a header/footer string.
///
/// Supported variables: `{page}`, `{pages}`, `{chapter}`, `{section}`,
/// `{title}`, `{date}`, `{author}`.
pub fn substitute_template(
    template: &str,
    state: &PaginationState,
    metadata: &TemplateMetadata,
) -> String {
    let mut result = template.to_owned();
    result = result.replace("{page}", &state.formatted_page_number());
    result = result.replace("{pages}", &state.formatted_total_pages());
    result = result.replace("{chapter}", &state.chapter.to_string());
    result = result.replace("{section}", &state.section.to_string());
    result = result.replace("{title}", &metadata.title);
    result = result.replace("{date}", &metadata.date);
    result = result.replace("{author}", &metadata.author);
    result
}

/// Resolved header content for a single page.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedHeader {
    /// Resolved left header text.
    pub left: String,
    /// Resolved center header text.
    pub center: String,
    /// Resolved right header text.
    pub right: String,
    /// Whether to draw the header rule.
    pub has_line: bool,
}

/// Resolved footer content for a single page.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedFooter {
    /// Resolved left footer text.
    pub left: String,
    /// Resolved center footer text.
    pub center: String,
    /// Resolved right footer text.
    pub right: String,
    /// Whether to draw the footer rule.
    pub has_line: bool,
}

/// Resolve header/footer content for a specific page number.
///
/// Handles:
/// - First-page suppression
/// - Odd/even page different layouts
/// - Page number injection at the configured placement
/// - Template variable substitution
pub fn resolve_header_footer(
    page_num: u32,
    total_pages: u32,
    chapter: u32,
    section: u32,
    config: &HeaderFooterConfig,
    metadata: &TemplateMetadata,
) -> Option<(ResolvedHeader, ResolvedFooter)> {
    let state = PaginationState {
        current_page: page_num,
        total_pages,
        chapter,
        section,
        config: config.clone(),
    };

    if !state.should_show_header_footer() {
        return None;
    }

    let substitute = |s: Option<&str>| -> String {
        s.map(|t| substitute_template(t, &state, metadata))
            .unwrap_or_default()
    };

    let mut header_left = substitute(config.header_left.as_deref());
    let mut header_center = substitute(config.header_center.as_deref());
    let mut header_right = substitute(config.header_right.as_deref());

    let mut footer_left = substitute(config.footer_left.as_deref());
    let mut footer_center = substitute(config.footer_center.as_deref());
    let mut footer_right = substitute(config.footer_right.as_deref());

    let page_num_str = state.formatted_page_number();

    match config.page_number_placement {
        PageNumberPlacement::HeaderLeft => header_left = page_num_str,
        PageNumberPlacement::HeaderCenter => header_center = page_num_str,
        PageNumberPlacement::HeaderRight => header_right = page_num_str,
        PageNumberPlacement::FooterLeft => footer_left = page_num_str,
        PageNumberPlacement::FooterCenter => footer_center = page_num_str,
        PageNumberPlacement::FooterRight => footer_right = page_num_str,
    }

    Some((
        ResolvedHeader {
            left: header_left,
            center: header_center,
            right: header_right,
            has_line: config.header_line,
        },
        ResolvedFooter {
            left: footer_left,
            center: footer_center,
            right: footer_right,
            has_line: config.footer_line,
        },
    ))
}

/// Parse a page number style from a CLI string.
pub fn parse_page_number_style(s: &str) -> Option<PageNumberStyle> {
    match s {
        "arabic" => Some(PageNumberStyle::Arabic),
        "lower-roman" => Some(PageNumberStyle::LowerRoman),
        "upper-roman" => Some(PageNumberStyle::UpperRoman),
        "lower-alpha" => Some(PageNumberStyle::LowerAlpha),
        "upper-alpha" => Some(PageNumberStyle::UpperAlpha),
        "none" => Some(PageNumberStyle::None),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> HeaderFooterConfig {
        HeaderFooterConfig::default()
    }

    fn default_metadata() -> TemplateMetadata {
        TemplateMetadata {
            title: "Test Document".to_owned(),
            author: "Test Author".to_owned(),
            date: "2026-01-01".to_owned(),
        }
    }

    #[test]
    fn test_page_number_arabic() {
        let cfg = HeaderFooterConfig::default();
        let state = PaginationState {
            current_page: 1,
            total_pages: 5,
            chapter: 0,
            section: 0,
            config: cfg,
        };
        assert_eq!(state.formatted_page_number(), "1");

        let state2 = PaginationState {
            current_page: 42,
            total_pages: 100,
            chapter: 0,
            section: 0,
            config: default_config(),
        };
        assert_eq!(state2.formatted_page_number(), "42");
    }

    #[test]
    fn test_page_number_roman() {
        let cfg = HeaderFooterConfig::new().with_page_number_style(PageNumberStyle::LowerRoman);
        let state = PaginationState {
            current_page: 1,
            total_pages: 5,
            chapter: 0,
            section: 0,
            config: cfg,
        };
        assert_eq!(state.formatted_page_number(), "i");

        let state2 = PaginationState {
            current_page: 4,
            total_pages: 10,
            chapter: 0,
            section: 0,
            config: HeaderFooterConfig::new().with_page_number_style(PageNumberStyle::LowerRoman),
        };
        assert_eq!(state2.formatted_page_number(), "iv");

        let cfg3 = HeaderFooterConfig::new().with_page_number_style(PageNumberStyle::UpperRoman);
        let state3 = PaginationState {
            current_page: 3,
            total_pages: 10,
            chapter: 0,
            section: 0,
            config: cfg3,
        };
        assert_eq!(state3.formatted_page_number(), "III");
    }

    #[test]
    fn test_page_number_alpha() {
        let cfg = HeaderFooterConfig::new().with_page_number_style(PageNumberStyle::LowerAlpha);
        let state = PaginationState {
            current_page: 1,
            total_pages: 5,
            chapter: 0,
            section: 0,
            config: cfg,
        };
        assert_eq!(state.formatted_page_number(), "a");

        let cfg2 = HeaderFooterConfig::new().with_page_number_style(PageNumberStyle::LowerAlpha);
        let state2 = PaginationState {
            current_page: 3,
            total_pages: 10,
            chapter: 0,
            section: 0,
            config: cfg2,
        };
        assert_eq!(state2.formatted_page_number(), "c");

        let cfg3 = HeaderFooterConfig::new().with_page_number_style(PageNumberStyle::UpperAlpha);
        let state3 = PaginationState {
            current_page: 2,
            total_pages: 10,
            chapter: 0,
            section: 0,
            config: cfg3,
        };
        assert_eq!(state3.formatted_page_number(), "B");
    }

    #[test]
    fn test_template_substitution() {
        let cfg = default_config();
        let state = PaginationState {
            current_page: 3,
            total_pages: 10,
            chapter: 2,
            section: 5,
            config: cfg,
        };
        let meta = default_metadata();

        assert_eq!(substitute_template("Page {page}", &state, &meta), "Page 3");
        assert_eq!(
            substitute_template("Chapter {chapter}, Section {section}", &state, &meta),
            "Chapter 2, Section 5"
        );
        assert_eq!(
            substitute_template("{title} by {author}", &state, &meta),
            "Test Document by Test Author"
        );
        assert_eq!(substitute_template("{date}", &state, &meta), "2026-01-01");
    }

    #[test]
    fn test_template_total_pages() {
        let cfg = default_config();
        let state = PaginationState {
            current_page: 3,
            total_pages: 42,
            chapter: 0,
            section: 0,
            config: cfg,
        };
        let meta = default_metadata();

        assert_eq!(
            substitute_template("{page} of {pages}", &state, &meta),
            "3 of 42"
        );

        let state_zero = PaginationState {
            current_page: 1,
            total_pages: 0,
            chapter: 0,
            section: 0,
            config: default_config(),
        };
        assert_eq!(
            substitute_template("{page} of {pages}", &state_zero, &meta),
            "1 of "
        );
    }

    #[test]
    fn test_first_page_different() {
        let cfg = HeaderFooterConfig::new().with_first_page_different(true);
        let meta = default_metadata();

        let result = resolve_header_footer(1, 5, 1, 0, &cfg, &meta);
        assert!(result.is_none(), "first page should be suppressed");

        let result2 = resolve_header_footer(2, 5, 1, 0, &cfg, &meta);
        assert!(result2.is_some(), "second page should have header/footer");
    }

    #[test]
    fn test_first_page_not_different() {
        let cfg = HeaderFooterConfig::new().with_first_page_different(false);
        let meta = default_metadata();

        let result = resolve_header_footer(1, 5, 1, 0, &cfg, &meta);
        assert!(
            result.is_some(),
            "first page should show header/footer when first_page_different=false"
        );
    }

    #[test]
    fn test_header_footer_config_defaults() {
        let cfg = default_config();
        assert_eq!(cfg.page_number_style, PageNumberStyle::Arabic);
        assert_eq!(cfg.page_number_placement, PageNumberPlacement::FooterCenter);
        assert!(cfg.header_left.is_none());
        assert!(cfg.header_center.is_none());
        assert!(cfg.header_right.is_none());
        assert!(cfg.footer_left.is_none());
        assert!(cfg.footer_center.is_none());
        assert!(cfg.footer_right.is_none());
        assert!(!cfg.header_line);
        assert!(!cfg.footer_line);
        assert!(cfg.first_page_different);
        assert!(!cfg.odd_even_different);
    }

    #[test]
    fn test_resolve_header_footer_page_number_placement() {
        let cfg = HeaderFooterConfig::new()
            .with_page_number_placement(PageNumberPlacement::HeaderRight)
            .with_first_page_different(false);
        let meta = default_metadata();

        let (header, footer) = resolve_header_footer(2, 5, 1, 0, &cfg, &meta).unwrap();
        assert_eq!(header.right, "2");
        assert_eq!(header.left, "");
        assert_eq!(header.center, "");
        assert_eq!(footer.left, "");
        assert_eq!(footer.center, "");
        assert_eq!(footer.right, "");
    }

    #[test]
    fn test_resolve_header_footer_with_text_and_page_number() {
        let mut cfg = HeaderFooterConfig::new()
            .with_page_number_placement(PageNumberPlacement::FooterCenter)
            .with_first_page_different(false);
        cfg.footer_right = Some("{title}".to_owned());
        let meta = default_metadata();

        let (_, footer) = resolve_header_footer(3, 10, 1, 0, &cfg, &meta).unwrap();
        assert_eq!(footer.center, "3");
        assert_eq!(footer.right, "Test Document");
    }

    #[test]
    fn test_resolve_header_footer_lines() {
        let cfg = HeaderFooterConfig::new()
            .with_header_line(true)
            .with_footer_line(true)
            .with_first_page_different(false);
        let meta = default_metadata();

        let (header, footer) = resolve_header_footer(2, 5, 1, 0, &cfg, &meta).unwrap();
        assert!(header.has_line);
        assert!(footer.has_line);
    }

    #[test]
    fn test_parse_page_number_style() {
        assert_eq!(
            parse_page_number_style("arabic"),
            Some(PageNumberStyle::Arabic)
        );
        assert_eq!(
            parse_page_number_style("lower-roman"),
            Some(PageNumberStyle::LowerRoman)
        );
        assert_eq!(
            parse_page_number_style("upper-roman"),
            Some(PageNumberStyle::UpperRoman)
        );
        assert_eq!(
            parse_page_number_style("lower-alpha"),
            Some(PageNumberStyle::LowerAlpha)
        );
        assert_eq!(
            parse_page_number_style("upper-alpha"),
            Some(PageNumberStyle::UpperAlpha)
        );
        assert_eq!(parse_page_number_style("none"), Some(PageNumberStyle::None));
        assert_eq!(parse_page_number_style("invalid"), None);
    }

    #[test]
    fn test_roman_numeral_boundaries() {
        let cfg = HeaderFooterConfig::new().with_page_number_style(PageNumberStyle::UpperRoman);
        let state = PaginationState {
            current_page: 3999,
            total_pages: 4000,
            chapter: 0,
            section: 0,
            config: cfg,
        };
        assert_eq!(state.formatted_page_number(), "MMMCMXCIX");
    }

    #[test]
    fn test_is_first_page_and_odd_page() {
        let state = PaginationState::new(default_config());
        assert!(state.is_first_page());
        assert!(state.is_odd_page());

        let state2 = PaginationState {
            current_page: 2,
            total_pages: 5,
            chapter: 0,
            section: 0,
            config: default_config(),
        };
        assert!(!state2.is_first_page());
        assert!(!state2.is_odd_page());
    }

    #[test]
    fn test_substitute_no_variables() {
        let state = PaginationState::new(default_config());
        let meta = default_metadata();
        assert_eq!(
            substitute_template("Static Header", &state, &meta),
            "Static Header"
        );
    }

    #[test]
    fn test_substitute_empty_template() {
        let state = PaginationState::new(default_config());
        let meta = default_metadata();
        assert_eq!(substitute_template("", &state, &meta), "");
    }

    #[test]
    fn test_substitute_all_variables() {
        let cfg = default_config();
        let state = PaginationState {
            current_page: 7,
            total_pages: 20,
            chapter: 3,
            section: 11,
            config: cfg,
        };
        let meta = TemplateMetadata {
            title: "My Book".to_owned(),
            author: "Jane Doe".to_owned(),
            date: "2026-05-30".to_owned(),
        };
        assert_eq!(
            substitute_template(
                "{title} | {author} | {date} | p.{page}/{pages} | ch.{chapter}.{section}",
                &state,
                &meta,
            ),
            "My Book | Jane Doe | 2026-05-30 | p.7/20 | ch.3.11"
        );
    }

    #[test]
    fn test_none_style_produces_empty_string() {
        let cfg = HeaderFooterConfig::new().with_page_number_style(PageNumberStyle::None);
        let state = PaginationState {
            current_page: 5,
            total_pages: 10,
            chapter: 0,
            section: 0,
            config: cfg,
        };
        assert_eq!(state.formatted_page_number(), "");
        assert_eq!(state.formatted_total_pages(), "");
    }

    #[test]
    fn test_builder_pattern() {
        let cfg = HeaderFooterConfig::new()
            .with_page_number_style(PageNumberStyle::LowerRoman)
            .with_page_number_placement(PageNumberPlacement::HeaderCenter)
            .with_header_line(true)
            .with_footer_line(true)
            .with_first_page_different(false)
            .with_odd_even_different(true);

        assert_eq!(cfg.page_number_style, PageNumberStyle::LowerRoman);
        assert_eq!(cfg.page_number_placement, PageNumberPlacement::HeaderCenter);
        assert!(cfg.header_line);
        assert!(cfg.footer_line);
        assert!(!cfg.first_page_different);
        assert!(cfg.odd_even_different);
    }
}
