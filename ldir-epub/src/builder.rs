use ldir_html::{HtmlOptions, HtmlRenderer, MathFormat};
use ldir_ir::sir::v2::module::SIRModuleV2;

pub struct EpubOptions {
    pub include_toc: bool,
    pub css: Option<String>,
}

impl Default for EpubOptions {
    fn default() -> Self {
        Self {
            include_toc: true,
            css: None,
        }
    }
}

pub struct EpubBuilder {
    options: EpubOptions,
}

impl Default for EpubBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EpubBuilder {
    pub fn new() -> Self {
        Self::with_options(EpubOptions::default())
    }

    pub fn with_options(options: EpubOptions) -> Self {
        Self { options }
    }

    pub fn build(&self, module: &SIRModuleV2) -> Result<Vec<u8>, String> {
        let title = module.metadata.title.as_deref().unwrap_or("Untitled");
        let author = module.metadata.author.as_deref().unwrap_or("Unknown");
        let lang = &module.metadata.language;
        let uid = format!("urn:uuid:{}", uuid_simple());

        let html_options = HtmlOptions {
            include_toc: self.options.include_toc,
            include_styles: false,
            math_format: MathFormat::LaTeX,
            indent: 2,
        };
        let content_html = HtmlRenderer::with_options(html_options).render(module);

        let body_content = extract_body(&content_html);

        let chapter_xhtml = format_xhtml(title, &body_content);
        let css = self.options.css.clone().unwrap_or_else(default_css);
        let opf = format_opf(title, author, lang, &uid);
        let toc_xhtml = format_toc_xhtml(title, module);
        let container_xml = format_container_xml();
        let toc_ncx = format_toc_ncx(title, &uid);

        let mut zip = SimpleZip::new();
        zip.add_file("mimetype", "application/epub+zip".as_bytes(), true);
        zip.add_file("META-INF/container.xml", container_xml.as_bytes(), false);
        zip.add_file("OEBPS/content.opf", opf.as_bytes(), false);
        zip.add_file("OEBPS/toc.ncx", toc_ncx.as_bytes(), false);
        zip.add_file("OEBPS/toc.xhtml", toc_xhtml.as_bytes(), false);
        zip.add_file("OEBPS/style.css", css.as_bytes(), false);
        zip.add_file("OEBPS/chapter1.xhtml", chapter_xhtml.as_bytes(), false);

        zip.finish()
    }
}

fn extract_body(html: &str) -> String {
    if let Some(start) = html.find("<body>") {
        let start = start + "<body>".len();
        if let Some(end) = html.find("</body>") {
            return html[start..end].to_string();
        }
    }
    html.to_string()
}

fn format_xhtml(title: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
<meta charset="UTF-8"/>
<title>{title}</title>
<link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
{body}
</body>
</html>"#,
        title = escape_xml(title),
        body = body
    )
}

fn default_css() -> String {
    r#"body { font-family: serif; line-height: 1.6; margin: 1em; }
h1, h2, h3, h4 { margin-top: 1.5em; margin-bottom: 0.5em; }
p { margin: 0.8em 0; text-align: justify; }
blockquote { border-left: 3px solid #ccc; margin-left: 0; padding-left: 1em; }
pre { background: #f5f5f5; padding: 1em; white-space: pre-wrap; }
code { font-family: monospace; }
table { border-collapse: collapse; width: 100%; }
th, td { border: 1px solid #ddd; padding: 0.5em; }
img { max-width: 100%; }
a { color: #0066cc; }
.math-display { display: block; text-align: center; margin: 1em 0; }
"#
    .to_string()
}

fn format_opf(title: &str, author: &str, lang: &str, uid: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="uid">{uid}</dc:identifier>
  <dc:title>{title}</dc:title>
  <dc:language>{lang}</dc:language>
  <dc:creator>{author}</dc:creator>
  <meta property="dcterms:modified">{date_now}</meta>
</metadata>
<manifest>
  <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  <item id="toc" href="toc.xhtml" media-type="application/xhtml+xml" properties="nav"/>
  <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  <item id="style" href="style.css" media-type="text/css"/>
</manifest>
<spine>
  <itemref idref="chapter1"/>
</spine>
</package>"#,
        title = escape_xml(title),
        author = escape_xml(author),
        lang = lang,
        uid = uid,
        date_now = date_now_str()
    )
}

fn format_toc_xhtml(title: &str, module: &SIRModuleV2) -> String {
    let mut items = String::new();
    for node in module.headings() {
        if let Some(_level) = node.heading_level() {
            let text = module.body.collect_text(node.id);
            let fallback_id = format!("heading-{}", node.id);
            let id = node.label.as_deref().unwrap_or(&fallback_id);
            items.push_str(&format!(
                "      <li><a href=\"chapter1.xhtml#{}\">{}</a></li>\n",
                id,
                escape_xml(&text)
            ));
        }
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><meta charset="UTF-8"/><title>{title}</title></head>
<body>
<nav epub:type="toc" id="toc">
  <h1>Table of Contents</h1>
  <ol>
{items}  </ol>
</nav>
</body>
</html>"#,
        title = escape_xml(title)
    )
}

fn format_toc_ncx(title: &str, uid: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
<head>
<meta name="dtb:uid" content="{uid}"/>
</head>
<docTitle><text>{title}</text></docTitle>
<navMap>
<navPoint id="nav-1" playOrder="1">
<navLabel><text>{title}</text></navLabel>
<content src="chapter1.xhtml"/>
</navPoint>
</navMap>
</ncx>"#,
        title = escape_xml(title),
        uid = uid
    )
}

fn format_container_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles>
<rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
</rootfiles>
</container>"#
        .to_string()
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", secs, 0, 0, 0, 0)
}

fn date_now_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let secs = d % 86400;
    let days_since_epoch = d / 86400;
    let (year, month, day) = days_to_date(days_since_epoch as i64);
    format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        month,
        day,
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

fn days_to_date(days: i64) -> (i64, i64, i64) {
    let year = 1970 + days / 365;
    let remaining = days % 365;
    let month = (remaining / 30) + 1;
    let day = (remaining % 30) + 1;
    (year.min(2100), month.min(12), day.min(31))
}

struct SimpleZip {
    files: Vec<(String, Vec<u8>, bool)>,
}

impl SimpleZip {
    fn new() -> Self {
        Self { files: Vec::new() }
    }

    fn add_file(&mut self, path: &str, data: &[u8], stored: bool) {
        self.files.push((path.to_string(), data.to_vec(), stored));
    }

    fn finish(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        let mut central_offsets: Vec<(u64, u32, u32, String)> = Vec::new();

        for (path, data, stored) in &self.files {
            let offset = buf.len() as u64;
            let crc = crc32(data);
            let size = data.len() as u32;

            buf.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
            buf.extend_from_slice(&[20, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(if *stored { &[0, 0] } else { &[8, 0] });
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&crc.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            let name_bytes = path.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(&[0u8; 10]);
            buf.extend_from_slice(name_bytes);
            buf.extend_from_slice(data);

            central_offsets.push((offset, size, crc, path.clone()));
        }

        let central_start = buf.len() as u64;

        for (offset, size, crc, name) in &central_offsets {
            buf.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
            buf.extend_from_slice(&[20, 0]);
            buf.extend_from_slice(&[20, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&crc.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            buf.extend_from_slice(&size.to_le_bytes());
            let name_bytes = name.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(&[0u8; 10]);
            buf.extend_from_slice(&[0, 0, 0]);
            buf.extend_from_slice(&(*offset as u32).to_le_bytes());
            buf.extend_from_slice(name_bytes);
        }

        let central_end = buf.len() as u64;
        let central_count = central_offsets.len() as u16;

        buf.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        buf.extend_from_slice(&[0, 0]);
        buf.extend_from_slice(&[0, 0]);
        buf.extend_from_slice(&central_count.to_le_bytes());
        buf.extend_from_slice(&central_count.to_le_bytes());
        buf.extend_from_slice(&(central_start as u32).to_le_bytes());
        buf.extend_from_slice(&(central_end as u32).to_le_bytes());
        buf.extend_from_slice(&[0u8; 2]);

        Ok(buf)
    }
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::*;

    fn make_simple_module() -> SIRModuleV2 {
        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Test Book".into());
        m.metadata.author = Some("Author".into());
        m.metadata.language = "en".into();
        m.body.push(Node::new(0, NodeType::Document));
        m.body.push(
            Node::new(1, NodeType::Section)
                .with_parent(0)
                .with_label("sec:1"),
        );
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Chapter 1".into(),
                },
            )
            .with_parent(1),
        );
        m.body
            .push(Node::new(3, NodeType::Paragraph).with_parent(0));
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Hello EPUB!".into(),
                },
            )
            .with_parent(3),
        );
        if let Some(node) = m.body.get_mut(0) { node.add_child(1); }
        if let Some(node) = m.body.get_mut(0) { node.add_child(3); }
        if let Some(node) = m.body.get_mut(1) { node.add_child(2); }
        if let Some(node) = m.body.get_mut(3) { node.add_child(4); }
        m
    }

    #[test]
    fn test_epub_builds() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        assert!(epub.len() > 100);
        Ok(())
    }

    #[test]
    fn test_epub_starts_with_pk() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        assert_eq!(&epub[0..4], b"PK\x03\x04");
        Ok(())
    }

    #[test]
    fn test_epub_contains_mimetype() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("application/epub+zip"));
        Ok(())
    }

    #[test]
    fn test_epub_contains_title() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("Test Book"));
        Ok(())
    }

    #[test]
    fn test_epub_contains_content() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("Hello EPUB!"));
        Ok(())
    }

    #[test]
    fn test_epub_has_container_xml() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("container.xml"));
        Ok(())
    }

    #[test]
    fn test_crc32() {
        assert_eq!(crc32(b""), 0x00000000);
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"q\""), "&quot;q&quot;");
    }

    #[test]
    fn test_epub_with_custom_css() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::with_options(EpubOptions {
            include_toc: false,
            css: Some("body { color: red; }".into()),
        })
        .build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("color: red"));
        Ok(())
    }

    #[test]
    fn test_epub_no_toc_when_disabled() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::with_options(EpubOptions {
            include_toc: false,
            css: None,
        })
        .build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(!text.contains("class=\"toc\""));
        Ok(())
    }

    #[test]
    fn test_epub_default_builder() -> Result<(), Box<dyn std::error::Error>> {
        let builder = EpubBuilder::default();
        let m = make_simple_module();
        let epub = builder.build(&m)?;
        assert!(epub.len() > 100);
        Ok(())
    }
}
