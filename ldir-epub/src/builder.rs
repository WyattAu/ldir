use std::time::Duration;

use ldir_html::{HtmlOptions, HtmlRenderer, MathFormat};
use ldir_ir::sir::v2::module::SIRModuleV2;

#[derive(Debug, Clone, thiserror::Error)]
pub enum EpubError {
    #[error("EPUB build error: {0}")]
    BuildError(String),
}

#[derive(Debug, Clone)]
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

/// A single text-audio synchronization pair within a media overlay `<par>` element.
#[derive(Debug, Clone)]
pub struct OverlayParam {
    pub text_ref: String,
    pub audio_ref: String,
    pub clip_begin: Option<f32>,
    pub clip_end: Option<f32>,
}

/// A media overlay for synchronized text and audio playback.
///
/// Each overlay corresponds to one SMIL document containing a `<body>` with `<par>` children.
#[derive(Debug, Clone)]
pub struct MediaOverlay {
    pub body_id: String,
    pub params: Vec<OverlayParam>,
}

#[derive(Debug, Clone)]
pub struct EpubBuilder {
    options: EpubOptions,
    overlays: Vec<MediaOverlay>,
    narrator: Option<String>,
    total_duration: Option<Duration>,
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
        Self {
            options,
            overlays: Vec::new(),
            narrator: None,
            total_duration: None,
        }
    }

    /// Add a media overlay (SMIL document) for synchronized text-audio playback.
    pub fn add_media_overlay(&mut self, overlay: MediaOverlay) -> Result<(), EpubError> {
        if overlay.body_id.is_empty() {
            return Err(EpubError::BuildError(
                "media overlay body_id must not be empty".into(),
            ));
        }
        self.overlays.push(overlay);
        Ok(())
    }

    /// Set the narrator name for media overlay metadata.
    pub fn set_narrator(&mut self, name: &str) {
        self.narrator = Some(name.to_string());
    }

    /// Set the total duration of all media overlays.
    pub fn set_total_duration(&mut self, duration: Duration) {
        self.total_duration = Some(duration);
    }

    #[must_use = "building EPUB can fail; check the result"]
    pub fn build(&self, module: &SIRModuleV2) -> Result<Vec<u8>, EpubError> {
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
        let opf = format_opf(
            title,
            author,
            lang,
            &uid,
            &self.overlays,
            &self.narrator,
            &self.total_duration,
        );
        let toc_xhtml = format_nav_xhtml(title, module);
        let container_xml = format_container_xml();
        let toc_ncx = format_toc_ncx(title, &uid);

        let mut zip = SimpleZip::new();
        let mut mimetype_bytes = b"application/epub+zip".to_vec();
        mimetype_bytes.push(0);
        zip.add_file("mimetype", &mimetype_bytes, true);
        zip.add_file("META-INF/container.xml", container_xml.as_bytes(), false);
        zip.add_file("OEBPS/content.opf", opf.as_bytes(), false);
        zip.add_file("OEBPS/toc.ncx", toc_ncx.as_bytes(), false);
        zip.add_file("OEBPS/toc.xhtml", toc_xhtml.as_bytes(), false);
        zip.add_file("OEBPS/style.css", css.as_bytes(), false);
        zip.add_file("OEBPS/chapter1.xhtml", chapter_xhtml.as_bytes(), false);

        for overlay in &self.overlays {
            let smil = format_smil(overlay);
            let path = format!("OEBPS/overlays/{}.smil", overlay.body_id);
            zip.add_file(&path, smil.as_bytes(), false);
        }

        zip.finish().map_err(EpubError::BuildError)
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

fn format_opf(
    title: &str,
    author: &str,
    lang: &str,
    uid: &str,
    overlays: &[MediaOverlay],
    narrator: &Option<String>,
    total_duration: &Option<Duration>,
) -> String {
    let mut manifest_extra = String::new();
    let mut spine_extra = String::new();
    let mut meta_extra = String::new();

    for overlay in overlays {
        let item_id = format!("{}-mo", overlay.body_id);
        let href = format!("overlays/{}.smil", overlay.body_id);
        manifest_extra.push_str(&format!(
            "  <item id=\"{item_id}\" href=\"{href}\" media-type=\"application/smil+xml\" properties=\"media-overlay\"/>\n",
            item_id = escape_xml(&item_id),
            href = escape_xml(&href),
        ));
        spine_extra.push_str(&format!(
            "  <itemref idref=\"{body_id}\" media-overlay=\"{item_id}\"/>\n",
            body_id = escape_xml(&overlay.body_id),
            item_id = escape_xml(&item_id),
        ));
    }

    if let Some(name) = narrator {
        meta_extra.push_str(&format!(
            "  <meta property=\"media:narrator\">{}</meta>\n",
            escape_xml(name),
        ));
    }
    if let Some(dur) = total_duration {
        meta_extra.push_str(&format!(
            "  <meta property=\"media:duration\">{}</meta>\n",
            format_duration(*dur),
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:identifier id="uid">{uid}</dc:identifier>
  <dc:title>{title}</dc:title>
  <dc:language>{lang}</dc:language>
  <dc:creator>{author}</dc:creator>
  <dc:date>{date_only}</dc:date>
  <meta property="dcterms:modified">{date_now}</meta>
  <meta property="a11y:validate">true</meta>
  <meta property="rendition:layout">reflowable</meta>
{meta_extra}</metadata>
<manifest>
  <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml" properties="scripted"/>
  <item id="toc" href="toc.xhtml" media-type="application/xhtml+xml" properties="nav"/>
  <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  <item id="style" href="style.css" media-type="text/css"/>
{manifest_extra}</manifest>
<spine toc="ncx">
  <itemref idref="chapter1"/>
{spine_extra}</spine>
</package>"#,
        title = escape_xml(title),
        author = escape_xml(author),
        lang = lang,
        uid = uid,
        date_only = date_now_str().split('T').next().unwrap_or(""),
        date_now = date_now_str(),
        meta_extra = meta_extra,
        manifest_extra = manifest_extra,
        spine_extra = spine_extra,
    )
}

fn format_nav_xhtml(title: &str, module: &SIRModuleV2) -> String {
    let toc_items = build_nested_toc(module);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
<meta charset="UTF-8"/>
<title>{title} - Table of Contents</title>
</head>
<body>
<nav epub:type="toc" id="toc">
  <h1>Table of Contents</h1>
  <ol>
{toc_items}  </ol>
</nav>
<nav epub:type="landmarks" id="landmarks">
  <h2>Guide</h2>
  <ol>
    <li><a epub:type="cover" href="chapter1.xhtml">Cover</a></li>
    <li><a epub:type="toc" href="toc.xhtml">Table of Contents</a></li>
  </ol>
</nav>
</body>
</html>"#,
        title = escape_xml(title)
    )
}

struct TocEntry {
    id: String,
    text: String,
    level: u8,
}

fn build_nested_toc(module: &SIRModuleV2) -> String {
    let mut entries: Vec<TocEntry> = Vec::new();
    for node in module.headings() {
        if let Some(level) = node.heading_level() {
            let text = module.body.collect_text(node.id);
            let fallback_id = format!("heading-{}", node.id);
            let id = node.label.as_deref().unwrap_or(&fallback_id);
            entries.push(TocEntry {
                id: id.to_string(),
                text,
                level,
            });
        }
    }
    format_nested_entries(&entries, 1)
}

fn format_nested_entries(entries: &[TocEntry], min_level: u8) -> String {
    let mut result = String::new();
    let mut current_level = min_level;
    for entry in entries {
        while entry.level > current_level {
            result.push_str("    ".repeat(current_level as usize).as_str());
            result.push_str("<ol>\n");
            current_level += 1;
        }
        while entry.level < current_level && current_level > min_level {
            current_level -= 1;
            result.push_str("    ".repeat(current_level as usize).as_str());
            result.push_str("</ol>\n");
        }
        result.push_str("    ".repeat(current_level as usize).as_str());
        result.push_str(&format!(
            "<li><a href=\"chapter1.xhtml#{}\">{}</a></li>\n",
            entry.id,
            escape_xml(&entry.text)
        ));
    }
    while current_level > min_level {
        current_level -= 1;
        result.push_str("    ".repeat(current_level as usize).as_str());
        result.push_str("</ol>\n");
    }
    result
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

fn format_smil(overlay: &MediaOverlay) -> String {
    let mut par_elements = String::new();
    for param in &overlay.params {
        let clip_begin = format_smil_timecode(param.clip_begin);
        let clip_end = format_smil_timecode(param.clip_end);
        par_elements.push_str(&format!(
            "    <par>\n      <text src=\"{}\"/>\n      <audio src=\"{}\" clipBegin=\"{}\" clipEnd=\"{}\"/>\n    </par>\n",
            escape_xml(&param.text_ref),
            escape_xml(&param.audio_ref),
            clip_begin,
            clip_end,
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<smil xmlns="http://www.w3.org/ns/SMIL" xmlns:epub="http://www.idpf.org/2007/ops" version="3.0">
<body>
<seq id="{body_id}" epub:textref="{body_id}.xhtml">
{par_elements}</seq>
</body>
</smil>"#,
        body_id = escape_xml(&overlay.body_id),
        par_elements = par_elements,
    )
}

fn format_smil_timecode(seconds: Option<f32>) -> String {
    match seconds {
        Some(s) => {
            let total_ms = (s * 1000.0).round() as u64;
            let ms = total_ms % 1000;
            let total_s = total_ms / 1000;
            let s_part = total_s % 60;
            let total_m = total_s / 60;
            let m_part = total_m % 60;
            let h_part = total_m / 60;
            format!("{:02}:{:02}:{:02}.{:03}", h_part, m_part, s_part, ms)
        }
        None => "0:00:00.000".to_string(),
    }
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs_f32();
    format_smil_timecode(Some(total_secs))
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
        if let Some(node) = m.body.get_mut(0) {
            node.add_child(1);
        }
        if let Some(node) = m.body.get_mut(0) {
            node.add_child(3);
        }
        if let Some(node) = m.body.get_mut(1) {
            node.add_child(2);
        }
        if let Some(node) = m.body.get_mut(3) {
            node.add_child(4);
        }
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
    fn test_epub_has_accessibility_meta() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("a11y:validate"));
        assert!(text.contains("rendition:layout"));
        assert!(text.contains("reflowable"));
        Ok(())
    }

    #[test]
    fn test_epub_has_landmarks_nav() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("epub:type=\"landmarks\""));
        assert!(text.contains("epub:type=\"cover\""));
        Ok(())
    }

    #[test]
    fn test_epub_has_dc_date() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("<dc:date>"));
        Ok(())
    }

    #[test]
    fn test_epub_mimetype_null_terminated() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let needle = b"application/epub+zip\x00";
        assert!(
            epub.windows(needle.len()).any(|w| w == needle),
            "mimetype file must be null-terminated per EPUB spec"
        );
        Ok(())
    }

    #[test]
    fn test_spine_has_toc_attribute() -> Result<(), Box<dyn std::error::Error>> {
        let m = make_simple_module();
        let epub = EpubBuilder::new().build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("spine toc=\"ncx\""));
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

    #[test]
    fn test_smil_generation() {
        let overlay = MediaOverlay {
            body_id: "chapter1".into(),
            params: vec![
                OverlayParam {
                    text_ref: "chapter1.xhtml#p1".into(),
                    audio_ref: "audio/chapter1.mp3".into(),
                    clip_begin: Some(0.0),
                    clip_end: Some(5.25),
                },
                OverlayParam {
                    text_ref: "chapter1.xhtml#p2".into(),
                    audio_ref: "audio/chapter1.mp3".into(),
                    clip_begin: Some(5.25),
                    clip_end: Some(12.75),
                },
            ],
        };
        let smil = format_smil(&overlay);
        assert!(smil.contains("xmlns=\"http://www.w3.org/ns/SMIL\""));
        assert!(smil.contains("epub:textref=\"chapter1.xhtml\""));
        assert!(smil.contains("<seq id=\"chapter1\""));
        assert!(smil.contains("<text src=\"chapter1.xhtml#p1\"/>"));
        assert!(smil.contains("<audio src=\"audio/chapter1.mp3\""));
        assert!(smil.contains("clipBegin=\"00:00:00.000\""));
        assert!(smil.contains("clipEnd=\"00:00:05.250\""));
        assert!(smil.contains("clipBegin=\"00:00:05.250\""));
        assert!(smil.contains("clipEnd=\"00:00:12.750\""));
    }

    #[test]
    fn test_overlay_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = EpubBuilder::new();
        builder.set_narrator("Jane Smith");
        builder.set_total_duration(Duration::from_secs(225));
        builder.add_media_overlay(MediaOverlay {
            body_id: "chapter1".into(),
            params: vec![OverlayParam {
                text_ref: "chapter1.xhtml#p1".into(),
                audio_ref: "audio/ch1.mp3".into(),
                clip_begin: Some(0.0),
                clip_end: Some(10.0),
            }],
        })?;
        let m = make_simple_module();
        let epub = builder.build(&m)?;
        let text = String::from_utf8_lossy(&epub);
        assert!(text.contains("media:narrator"));
        assert!(text.contains("Jane Smith"));
        assert!(text.contains("media:duration"));
        assert!(text.contains("overlays/chapter1.smil"));
        assert!(text.contains("media-overlay=\"chapter1-mo\""));
        assert!(text.contains("properties=\"media-overlay\""));
        Ok(())
    }

    #[test]
    fn test_overlay_duration_format() {
        assert_eq!(
            format_duration(Duration::from_secs(225) + Duration::from_millis(0)),
            "00:03:45.000"
        );
        assert_eq!(format_duration(Duration::from_secs(0)), "00:00:00.000");
        assert_eq!(
            format_duration(Duration::from_secs(3661) + Duration::from_millis(500)),
            "01:01:01.500"
        );
    }

    #[test]
    fn test_add_overlay_rejects_empty_id() {
        let mut builder = EpubBuilder::new();
        let result = builder.add_media_overlay(MediaOverlay {
            body_id: String::new(),
            params: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_smil_timecode_none() {
        assert_eq!(format_smil_timecode(None), "0:00:00.000");
    }

    #[test]
    fn test_smil_timecode_rounding() {
        assert_eq!(format_smil_timecode(Some(5.9999)), "00:00:06.000");
        assert_eq!(format_smil_timecode(Some(5.4999)), "00:00:05.500");
    }
}
