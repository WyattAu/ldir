/// Test plugin: Paragraph style application.
/// Finds paragraphs matching selectors and applies style properties.
pub struct ParagraphStylePlugin;

#[derive(Debug, Clone, Default)]
/// Style properties a selector may override; `None` leaves a property unchanged.
pub struct StyleOverride {
    /// Override for the font size.
    pub font_size: Option<f32>,
    /// Override for the line height.
    pub line_height: Option<f32>,
    /// Override for paragraph alignment.
    pub text_align: Option<TextAlign>,
    /// Override for the text color (CSS string).
    pub color: Option<String>,
    /// Override for the top margin.
    pub margin_top: Option<f32>,
    /// Override for the bottom margin.
    pub margin_bottom: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Horizontal paragraph alignment.
pub enum TextAlign {
    #[default]
    /// Flush left (ragged right).
    Left,
    /// Centered.
    Center,
    /// Flush right (ragged left).
    Right,
    /// Fully justified with stretched inter-word space.
    Justify,
}

impl ParagraphStylePlugin {
    /// Creates the plugin.
    pub fn new() -> Self {
        Self
    }

    /// Returns indices of paragraphs matching `selector`; only `"first"` is supported.
    pub fn apply_style(
        &self,
        selector: &str,
        _style: &StyleOverride,
        paragraph_count: usize,
    ) -> Vec<usize> {
        match selector {
            "first" if paragraph_count > 0 => vec![0],
            _ => Vec::new(),
        }
    }
}

impl Default for ParagraphStylePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraph_style_plugin_creation() {
        let plugin = ParagraphStylePlugin::new();
        let style = StyleOverride {
            font_size: Some(14.0),
            ..Default::default()
        };
        let matched = plugin.apply_style("first", &style, 5);
        assert_eq!(matched, vec![0]);
    }

    #[test]
    fn test_style_override_fields() {
        let style = StyleOverride {
            font_size: Some(12.0),
            line_height: Some(1.5),
            text_align: Some(TextAlign::Center),
            color: Some("#000000".to_string()),
            margin_top: Some(8.0),
            margin_bottom: Some(8.0),
        };
        assert_eq!(style.font_size, Some(12.0));
        assert_eq!(style.line_height, Some(1.5));
        assert_eq!(style.text_align, Some(TextAlign::Center));
        assert_eq!(style.color.as_deref(), Some("#000000"));
        assert_eq!(style.margin_top, Some(8.0));
        assert_eq!(style.margin_bottom, Some(8.0));
    }

    #[test]
    fn test_style_override_default() {
        let style = StyleOverride::default();
        assert!(style.font_size.is_none());
        assert!(style.line_height.is_none());
        assert_eq!(style.text_align, None);
        assert!(style.color.is_none());
        assert!(style.margin_top.is_none());
        assert!(style.margin_bottom.is_none());
    }

    #[test]
    fn test_no_match_unknown_selector() {
        let plugin = ParagraphStylePlugin::new();
        let style = StyleOverride::default();
        let matched = plugin.apply_style("unknown", &style, 5);
        assert!(matched.is_empty());
    }

    #[test]
    fn test_no_match_empty_paragraphs() {
        let plugin = ParagraphStylePlugin::new();
        let style = StyleOverride::default();
        let matched = plugin.apply_style("first", &style, 0);
        assert!(matched.is_empty());
    }
}
