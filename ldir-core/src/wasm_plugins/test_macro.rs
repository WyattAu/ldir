/// Test plugin: Simple macro expansion.
/// Replaces `{{macro_name}}` with predefined values.
pub struct MacroExpansionPlugin;

impl MacroExpansionPlugin {
    pub fn new() -> Self {
        Self
    }

    pub fn expand(&self, text: &str) -> String {
        let macros = [
            ("{{greeting}}", "Hello, World!"),
            ("{{date}}", "2025-01-15"),
            ("{{version}}", "0.1.0"),
            ("{{project}}", "ldir"),
        ];
        let mut result = text.to_string();
        for (pattern, replacement) in &macros {
            result = result.replace(pattern, replacement);
        }
        result
    }
}

impl Default for MacroExpansionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_greeting() {
        let plugin = MacroExpansionPlugin::new();
        assert_eq!(plugin.expand("{{greeting}}"), "Hello, World!");
    }

    #[test]
    fn test_multiple_macros() {
        let plugin = MacroExpansionPlugin::new();
        let result = plugin.expand("# {{greeting}}\n{{project}} v{{version}}");
        assert_eq!(result, "# Hello, World!\nldir v0.1.0");
    }

    #[test]
    fn test_no_macros() {
        let plugin = MacroExpansionPlugin::new();
        assert_eq!(plugin.expand("plain text"), "plain text");
    }

    #[test]
    fn test_macro_expansion_integration() {
        let plugin = MacroExpansionPlugin::new();
        let input = "# {{greeting}}\n\nThis is {{project}} version {{version}} ({{date}}).\n\n## Section\n\nSome content.";
        let output = plugin.expand(input);
        assert!(output.starts_with("# Hello, World!"));
        assert!(output.contains("ldir version 0.1.0"));
        assert!(output.contains("2025-01-15"));
        assert!(output.contains("## Section"));
        assert!(output.contains("Some content."));
        assert!(!output.contains("{{"));
        assert!(!output.contains("}}"));
    }

    #[test]
    fn test_default() {
        let _plugin = MacroExpansionPlugin::default();
    }
}
