use crate::conformance::PdfConformance;

/// Generates an XMP packet declaring PDF/A identification (`pdfaid:part`, `pdfaid:conformance`) plus Dublin Core title and author.
pub fn generate_pdfa_xmp(conformance: PdfConformance, title: &str, author: &str) -> Vec<u8> {
    let part = conformance.pdfaid_part();
    let conf = conformance.pdfaid_conformance();

    let mut xmp = format!(
        r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
           xmlns:dc="http://purl.org/dc/elements/1.1/"
           xmlns:xmp="http://ns.adobe.com/xap/1.0/"
           xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">
    <rdf:Description rdf:about=""
      xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">
      <pdfaid:part>{part}</pdfaid:part>"#
    );

    if !conf.is_empty() {
        xmp.push_str(&format!(
            r#"
      <pdfaid:conformance>{conf}</pdfaid:conformance>"#
        ));
    }

    xmp.push_str(
        r#"
    </rdf:Description>
    <rdf:Description rdf:about=""
      xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">"#,
    );
    xmp.push_str(&xml_escape(title));
    xmp.push_str(
        r#"</rdf:li>
        </rdf:Alt>
      </dc:title>
      <dc:creator>
        <rdf:Seq>
          <rdf:li>"#,
    );
    xmp.push_str(&xml_escape(author));
    xmp.push_str(
        r#"</rdf:li>
        </rdf:Seq>
      </dc:creator>
    </rdf:Description>
    <rdf:Description rdf:about=""
      xmlns:xmp="http://ns.adobe.com/xap/1.0/">
      <xmp:CreatorTool>ldir</xmp:CreatorTool>
      <xmp:CreateDate>"#,
    );
    xmp.push_str(&iso8601_now());
    xmp.push_str(
        r#"</xmp:CreateDate>
      <xmp:ModifyDate>"#,
    );
    xmp.push_str(&iso8601_now());
    xmp.push_str(
        r#"</xmp:ModifyDate>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
    );

    xmp.into_bytes()
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

fn iso8601_now() -> String {
    let epoch = std::time::SystemTime::UNIX_EPOCH;
    std::time::SystemTime::now()
        .duration_since(epoch)
        .map(|d| {
            let secs = d.as_secs();
            let days = secs / 86400;
            let z = (days + 719528) as i64;
            let era = if z >= 0 {
                z / 146097
            } else {
                (z - 146096) / 146097
            };
            let doe = z - era * 146097;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = yoe + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let y = if m <= 2 { y + 1 } else { y };
            format!("{:04}-{:02}-{:02}T00:00:00Z", y, m, d)
        })
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmp_pdfa2b_contains_part_and_conformance() {
        let xmp = generate_pdfa_xmp(PdfConformance::PdfA2b, "Test", "Author");
        let s = String::from_utf8_lossy(&xmp);
        assert!(s.contains("<pdfaid:part>2</pdfaid:part>"));
        assert!(s.contains("<pdfaid:conformance>B</pdfaid:conformance>"));
    }

    #[test]
    fn test_xmp_pdfa4_contains_part() {
        let xmp = generate_pdfa_xmp(PdfConformance::PdfA4, "Test", "Author");
        let s = String::from_utf8_lossy(&xmp);
        assert!(s.contains("<pdfaid:part>4</pdfaid:part>"));
        assert!(!s.contains("<pdfaid:conformance>"));
    }

    #[test]
    fn test_xmp_pdfa3b_contains_part_and_conformance() {
        let xmp = generate_pdfa_xmp(PdfConformance::PdfA3b, "Test", "Author");
        let s = String::from_utf8_lossy(&xmp);
        assert!(s.contains("<pdfaid:part>3</pdfaid:part>"));
        assert!(s.contains("<pdfaid:conformance>B</pdfaid:conformance>"));
    }

    #[test]
    fn test_xmp_contains_title_and_creator() {
        let xmp = generate_pdfa_xmp(PdfConformance::PdfA2b, "My Title", "Jane Doe");
        let s = String::from_utf8_lossy(&xmp);
        assert!(s.contains("My Title"));
        assert!(s.contains("Jane Doe"));
    }

    #[test]
    fn test_xmp_contains_dates() {
        let xmp = generate_pdfa_xmp(PdfConformance::PdfA2b, "Test", "Author");
        let s = String::from_utf8_lossy(&xmp);
        assert!(s.contains("<xmp:CreateDate>"));
        assert!(s.contains("<xmp:ModifyDate>"));
    }

    #[test]
    fn test_xmp_xml_escape() {
        let xmp = generate_pdfa_xmp(PdfConformance::PdfA2b, "A & B < C", "D");
        let s = String::from_utf8_lossy(&xmp);
        assert!(s.contains("A &amp; B &lt; C"));
    }

    #[test]
    fn test_xmp_xpacket_wrappers() {
        let xmp = generate_pdfa_xmp(PdfConformance::PdfA2b, "T", "A");
        let s = String::from_utf8_lossy(&xmp);
        assert!(s.contains("<?xpacket begin"));
        assert!(s.contains("<?xpacket end"));
    }

    #[test]
    fn test_iso8601_format() {
        let ts = iso8601_now();
        assert!(ts.len() == 20, "expected YYYY-MM-DDTHH:MM:SSZ, got: {}", ts);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert!(ts.ends_with("Z"));
    }
}
