//! Page template system for headers and footers.

#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

#[derive(Debug, Clone)]
pub struct PageTemplate {
    pub header_left: String,
    pub header_center: String,
    pub header_right: String,
    pub footer_left: String,
    pub footer_center: String,
    pub footer_right: String,
    pub header_rule: bool,
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

pub struct TemplateContext {
    pub page: usize,
    pub pages: usize,
    pub title: String,
    pub chapter: String,
    pub section: String,
    pub date: String,
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
    pub fn expand_header(&self, ctx: &TemplateContext) -> (String, String, String) {
        (
            expand_template(&self.header_left, ctx),
            expand_template(&self.header_center, ctx),
            expand_template(&self.header_right, ctx),
        )
    }

    pub fn expand_footer(&self, ctx: &TemplateContext) -> (String, String, String) {
        (
            expand_template(&self.footer_left, ctx),
            expand_template(&self.footer_center, ctx),
            expand_template(&self.footer_right, ctx),
        )
    }
}

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
}
