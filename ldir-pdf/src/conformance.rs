/// PDF/A conformance level.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfConformance {
    #[default]
    PdfA4,
    PdfA2b,
    PdfA3b,
}

impl PdfConformance {
    pub fn pdf_version_str(self) -> &'static str {
        match self {
            Self::PdfA4 => "2.0",
            Self::PdfA2b | Self::PdfA3b => "1.7",
        }
    }

    pub fn pdfaid_part(self) -> u8 {
        match self {
            Self::PdfA4 => 4,
            Self::PdfA2b => 2,
            Self::PdfA3b => 3,
        }
    }

    pub fn pdfaid_conformance(self) -> &'static str {
        match self {
            Self::PdfA4 => "",
            Self::PdfA2b => "B",
            Self::PdfA3b => "B",
        }
    }
}

impl std::fmt::Display for PdfConformance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PdfA4 => write!(f, "PDF/A-4"),
            Self::PdfA2b => write!(f, "PDF/A-2b"),
            Self::PdfA3b => write!(f, "PDF/A-3b"),
        }
    }
}

impl std::str::FromStr for PdfConformance {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "4" | "pdfa4" | "pdf/a-4" => Ok(Self::PdfA4),
            "2b" | "pdfa2b" | "pdf/a-2b" => Ok(Self::PdfA2b),
            "3b" | "pdfa3b" | "pdf/a-3b" => Ok(Self::PdfA3b),
            _ => Err(format!("unknown PDF/A level: {s}. Supported: 4, 2b, 3b")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conformance_pdf_version() {
        assert_eq!(PdfConformance::PdfA4.pdf_version_str(), "2.0");
        assert_eq!(PdfConformance::PdfA2b.pdf_version_str(), "1.7");
        assert_eq!(PdfConformance::PdfA3b.pdf_version_str(), "1.7");
    }

    #[test]
    fn test_conformance_pdfaid_part() {
        assert_eq!(PdfConformance::PdfA4.pdfaid_part(), 4);
        assert_eq!(PdfConformance::PdfA2b.pdfaid_part(), 2);
        assert_eq!(PdfConformance::PdfA3b.pdfaid_part(), 3);
    }

    #[test]
    fn test_conformance_pdfaid_conformance() {
        assert_eq!(PdfConformance::PdfA4.pdfaid_conformance(), "");
        assert_eq!(PdfConformance::PdfA2b.pdfaid_conformance(), "B");
        assert_eq!(PdfConformance::PdfA3b.pdfaid_conformance(), "B");
    }

    #[test]
    fn test_conformance_display() {
        assert_eq!(format!("{}", PdfConformance::PdfA4), "PDF/A-4");
        assert_eq!(format!("{}", PdfConformance::PdfA2b), "PDF/A-2b");
        assert_eq!(format!("{}", PdfConformance::PdfA3b), "PDF/A-3b");
    }

    #[test]
    fn test_conformance_from_str() {
        assert_eq!(
            "4".parse::<PdfConformance>().unwrap(),
            PdfConformance::PdfA4
        );
        assert_eq!(
            "2b".parse::<PdfConformance>().unwrap(),
            PdfConformance::PdfA2b
        );
        assert_eq!(
            "3b".parse::<PdfConformance>().unwrap(),
            PdfConformance::PdfA3b
        );
        assert_eq!(
            "pdfa2b".parse::<PdfConformance>().unwrap(),
            PdfConformance::PdfA2b
        );
        assert!("invalid".parse::<PdfConformance>().is_err());
    }

    #[test]
    fn test_conformance_default() {
        assert_eq!(PdfConformance::default(), PdfConformance::PdfA4);
    }
}
