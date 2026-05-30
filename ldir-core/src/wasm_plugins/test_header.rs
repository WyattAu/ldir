/// Test plugin: Page header injection.
/// Adds a running header to each page with configurable content.
pub struct PageHeaderPlugin;

#[derive(Debug, Clone)]
pub struct HeaderConfig {
    pub left: String,
    pub center: String,
    pub right: String,
    pub line: bool,
    pub first_page: bool,
}

impl Default for HeaderConfig {
    fn default() -> Self {
        Self {
            left: String::new(),
            center: String::new(),
            right: "{page}".to_string(),
            line: true,
            first_page: false,
        }
    }
}

impl PageHeaderPlugin {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_header(&self, config: &HeaderConfig, page: u32, chapter: &str) -> String {
        let center = config.center.replace("{chapter}", chapter);
        let right = config.right.replace("{page}", &page.to_string());

        if config.line {
            format!("{}\t{}\t{}\n\u{2500}", config.left, center, right)
        } else {
            format!("{}\t{}\t{}", config.left, center, right)
        }
    }
}

impl Default for PageHeaderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_header() {
        let plugin = PageHeaderPlugin::new();
        let config = HeaderConfig::default();
        let header = plugin.generate_header(&config, 1, "Chapter 1");
        assert!(header.contains("1"));
        assert!(header.contains('\u{2500}'));
    }

    #[test]
    fn test_custom_header() {
        let plugin = PageHeaderPlugin::new();
        let config = HeaderConfig {
            left: "ldir".to_string(),
            center: "{chapter}".to_string(),
            right: "{page}/{pages}".to_string(),
            line: false,
            first_page: true,
        };
        let header = plugin.generate_header(&config, 3, "Introduction");
        assert_eq!(header, "ldir\tIntroduction\t3/{pages}");
    }

    #[test]
    fn test_first_page_suppressed() {
        let plugin = PageHeaderPlugin::new();
        let config = HeaderConfig::default();
        let header = plugin.generate_header(&config, 1, "Preface");
        assert!(header.contains("1"));
    }

    #[test]
    fn test_header_plugin_no_panics() {
        let plugin = PageHeaderPlugin::new();
        let config = HeaderConfig {
            left: String::new(),
            center: String::new(),
            right: String::new(),
            line: false,
            first_page: true,
        };
        let header = plugin.generate_header(&config, 0, "");
        assert_eq!(header, "\t\t");
    }

    #[test]
    fn test_header_config_default() {
        let config = HeaderConfig::default();
        assert!(config.left.is_empty());
        assert!(config.center.is_empty());
        assert_eq!(config.right, "{page}");
        assert!(config.line);
        assert!(!config.first_page);
    }

    #[test]
    fn test_header_with_chapter_substitution() {
        let plugin = PageHeaderPlugin::new();
        let config = HeaderConfig {
            center: "Chapter: {chapter}".to_string(),
            ..Default::default()
        };
        let header = plugin.generate_header(&config, 5, "Advanced Topics");
        assert!(header.contains("Chapter: Advanced Topics"));
        assert!(header.contains("5"));
    }
}
