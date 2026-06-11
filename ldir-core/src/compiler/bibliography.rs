//! Bibliography resolver: loads .bib files and resolves citations to formatted strings.
//!
//! Provides [`BibliographyResolver`] which bridges the BibTeX parser with
//! citation formatters, supporting IEEE, APA, Chicago, and MLA styles.

#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;

use crate::compiler::bibtex::{
    BibEntry, format_citation_apa, format_citation_chicago, format_citation_ieee,
    format_citation_mla, parse_bib,
};

pub struct BibliographyResolver {
    entries: HashMap<String, BibEntry>,
    citation_style: String,
}

impl BibliographyResolver {
    pub fn new(style: &str) -> Self {
        Self {
            entries: HashMap::new(),
            citation_style: style.to_lowercase(),
        }
    }

    pub fn load_bib_file(&mut self, path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let entries = parse_bib(&content)?;
        for (key, entry) in entries {
            self.entries.insert(key, entry);
        }
        Ok(())
    }

    pub fn load_bib_content(&mut self, content: &str) -> Result<(), String> {
        let entries = parse_bib(content)?;
        for (key, entry) in entries {
            self.entries.insert(key, entry);
        }
        Ok(())
    }

    pub fn resolve_citation(&self, key: &str) -> Option<String> {
        let _entry = self.entries.get(key)?;
        let formatted = match self.citation_style.as_str() {
            "ieee" => format_citation_ieee(_entry),
            "apa" => format_citation_apa(_entry),
            "chicago" => format_citation_chicago(_entry),
            "mla" => format_citation_mla(_entry),
            _ => format_citation_ieee(_entry),
        };
        Some(formatted)
    }

    pub fn generate_bibliography(&self, cited_keys: &[String]) -> String {
        let mut seen: std::collections::HashSet<&String> = std::collections::HashSet::new();
        let mut unique_keys: Vec<&String> = Vec::new();
        for key in cited_keys {
            if seen.insert(key) {
                unique_keys.push(key);
            }
        }

        let mut numbered: Vec<(&String, &BibEntry)> = Vec::new();
        for key in &unique_keys {
            if let Some(entry) = self.entries.get(*key) {
                numbered.push((key, entry));
            }
        }

        if numbered.is_empty() {
            return String::new();
        }

        let mut lines: Vec<String> = Vec::with_capacity(numbered.len() + 1);
        lines.push("References".to_string());
        lines.push(String::new());

        for (i, (_key, entry)) in numbered.iter().enumerate() {
            let num = i + 1;
            let formatted = match self.citation_style.as_str() {
                "ieee" => format_citation_ieee(entry),
                "apa" => format_citation_apa(entry),
                "chicago" => format_citation_chicago(entry),
                "mla" => format_citation_mla(entry),
                _ => format_citation_ieee(entry),
            };
            lines.push(format!("[{}] {}", num, formatted));
            if i < numbered.len() - 1 {
                lines.push(String::new());
            }
        }

        lines.join("\n")
    }

    pub fn style(&self) -> &str {
        &self.citation_style
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_BIB: &str = r#"
@article{knuth1984,
  author = {Donald E. Knuth},
  title = {Literate Programming},
  journal = {The Computer Journal},
  volume = {27},
  number = {2},
  pages = {97--111},
  year = {1984},
}

@book{lamport1994,
  author = {Leslie Lamport},
  title = {{\LaTeX}: A Document Preparation System},
  publisher = {Addison-Wesley},
  year = {1994},
}

@inproceedings{dijkstra1968,
  author = {Edsger W. Dijkstra},
  title = {Go To Statement Considered Harmful},
  booktitle = {Proc. IFIP Congress},
  pages = {1--5},
  year = {1968},
}

@misc{rust2024,
  author = {The Rust Project Developers},
  title = {The Rust Programming Language},
  year = {2024},
}
"#;

    #[test]
    fn test_citation_resolution_ieee() {
        let mut resolver = BibliographyResolver::new("ieee");
        resolver.load_bib_content(TEST_BIB).unwrap();

        let result = resolver.resolve_citation("knuth1984");
        assert!(result.is_some());
        let formatted = result.unwrap();
        assert!(formatted.contains("Donald E. Knuth"));
        assert!(formatted.contains("Literate Programming"));
    }

    #[test]
    fn test_citation_resolution_mla() {
        let mut resolver = BibliographyResolver::new("mla");
        resolver.load_bib_content(TEST_BIB).unwrap();

        let result = resolver.resolve_citation("knuth1984");
        assert!(result.is_some());
        let formatted = result.unwrap();
        assert!(formatted.contains("Knuth, Donald"));
    }

    #[test]
    fn test_bibliography_generation() {
        let mut resolver = BibliographyResolver::new("ieee");
        resolver.load_bib_content(TEST_BIB).unwrap();

        let cited_keys = vec![
            "knuth1984".to_string(),
            "lamport1994".to_string(),
            "knuth1984".to_string(),
        ];
        let bib = resolver.generate_bibliography(&cited_keys);

        assert!(bib.contains("References"));
        assert!(bib.contains("[1]"));
        assert!(bib.contains("[2]"));
        assert!(bib.contains("Donald E. Knuth"));
        assert!(bib.contains("Leslie Lamport"));
    }

    #[test]
    fn test_missing_citation() {
        let mut resolver = BibliographyResolver::new("ieee");
        resolver.load_bib_content(TEST_BIB).unwrap();

        let result = resolver.resolve_citation("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_bibliography() {
        let mut resolver = BibliographyResolver::new("ieee");
        resolver.load_bib_content("").unwrap();

        let cited_keys: Vec<String> = vec![];
        let bib = resolver.generate_bibliography(&cited_keys);
        assert!(bib.is_empty());
    }

    #[test]
    fn test_resolver_style() {
        let resolver = BibliographyResolver::new("APA");
        assert_eq!(resolver.style(), "apa");

        let resolver = BibliographyResolver::new("MLA");
        assert_eq!(resolver.style(), "mla");
    }

    #[test]
    fn test_entry_count() {
        let mut resolver = BibliographyResolver::new("ieee");
        resolver.load_bib_content(TEST_BIB).unwrap();
        assert_eq!(resolver.entry_count(), 4);
    }

    #[test]
    fn test_contains_key() {
        let mut resolver = BibliographyResolver::new("ieee");
        resolver.load_bib_content(TEST_BIB).unwrap();
        assert!(resolver.contains_key("knuth1984"));
        assert!(!resolver.contains_key("missing"));
    }

    #[test]
    fn test_bibliography_generation_mla() {
        let mut resolver = BibliographyResolver::new("mla");
        resolver.load_bib_content(TEST_BIB).unwrap();

        let cited_keys = vec!["lamport1994".to_string()];
        let bib = resolver.generate_bibliography(&cited_keys);

        assert!(bib.contains("References"));
        assert!(bib.contains("[1]"));
        assert!(bib.contains("Lamport, Leslie"));
    }
}
