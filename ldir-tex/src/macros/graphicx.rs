use super::MacroRegistry;

pub fn register(_registry: &mut MacroRegistry) {}

#[derive(Debug, Clone, Default)]
pub struct GraphicsOptions {
    pub width: Option<String>,
    pub height: Option<String>,
    pub scale: Option<String>,
    pub angle: Option<String>,
    pub trim: Option<String>,
    pub clip: bool,
}

impl GraphicsOptions {
    pub fn parse(opts_str: &str) -> Self {
        let mut opts = Self::default();
        for part in opts_str.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((key, val)) = part.split_once('=') {
                match key.trim() {
                    "width" => opts.width = Some(val.trim().to_string()),
                    "height" => opts.height = Some(val.trim().to_string()),
                    "scale" => opts.scale = Some(val.trim().to_string()),
                    "angle" => opts.angle = Some(val.trim().to_string()),
                    "trim" => opts.trim = Some(val.trim().to_string()),
                    "clip" => opts.clip = true,
                    _ => {}
                }
            } else if part == "clip" {
                opts.clip = true;
            }
        }
        opts
    }

    pub fn format_suffix(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref w) = self.width {
            parts.push(format!("width={}", w));
        }
        if let Some(ref h) = self.height {
            parts.push(format!("height={}", h));
        }
        if let Some(ref s) = self.scale {
            parts.push(format!("scale={}", s));
        }
        if let Some(ref a) = self.angle {
            parts.push(format!("angle={}", a));
        }
        if let Some(ref t) = self.trim {
            parts.push(format!("trim={}", t));
        }
        if self.clip {
            parts.push("clip".to_string());
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("|{}", parts.join(","))
        }
    }
}

pub fn parse_graphics_paths(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            i += 1;
            let mut path = String::new();
            while i < chars.len() && chars[i] != '}' {
                path.push(chars[i]);
                i += 1;
            }
            let trimmed = path.trim().to_string();
            if !trimmed.is_empty() {
                paths.push(trimmed);
            }
            if i < chars.len() {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_options_empty() {
        let opts = GraphicsOptions::parse("");
        assert!(opts.width.is_none());
        assert!(opts.height.is_none());
        assert!(!opts.clip);
    }

    #[test]
    fn test_parse_options_width() {
        let opts = GraphicsOptions::parse("width=5cm");
        assert_eq!(opts.width.as_deref(), Some("5cm"));
    }

    #[test]
    fn test_parse_options_multiple() {
        let opts = GraphicsOptions::parse("width=5cm,height=3cm,scale=0.5,clip");
        assert_eq!(opts.width.as_deref(), Some("5cm"));
        assert_eq!(opts.height.as_deref(), Some("3cm"));
        assert_eq!(opts.scale.as_deref(), Some("0.5"));
        assert!(opts.clip);
    }

    #[test]
    fn test_parse_options_clip_alone() {
        let opts = GraphicsOptions::parse("clip");
        assert!(opts.clip);
    }

    #[test]
    fn test_format_suffix_empty() {
        let opts = GraphicsOptions::default();
        assert!(opts.format_suffix().is_empty());
    }

    #[test]
    fn test_format_suffix_with_options() {
        let mut opts = GraphicsOptions::default();
        opts.width = Some("5cm".to_string());
        let suffix = opts.format_suffix();
        assert!(suffix.contains("width=5cm"));
        assert!(suffix.starts_with('|'));
    }

    #[test]
    fn test_parse_graphics_paths_basic() {
        let paths = parse_graphics_paths("{images}{figures}");
        assert_eq!(paths, vec!["images", "figures"]);
    }

    #[test]
    fn test_parse_graphics_paths_empty() {
        let paths = parse_graphics_paths("");
        assert!(paths.is_empty());
    }

    #[test]
    fn test_parse_graphics_paths_single() {
        let paths = parse_graphics_paths("{assets/img}");
        assert_eq!(paths, vec!["assets/img"]);
    }
}
