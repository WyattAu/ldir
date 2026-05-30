//! BibTeX bibliography parser and IEEE/APA/Chicago citation formatter.
//!
//! Parses `.bib` files into structured [`BibEntry`] values and formats
//! in-text citations and bibliography entries according to IEEE, APA,
//! and Chicago (author-date) style conventions, with optional year
//! disambiguation for entries sharing the same author and year.

#![deny(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BibEntry {
    pub entry_type: String,
    pub key: String,
    pub fields: HashMap<String, String>,
}

pub fn parse_bib(bib_content: &str) -> Result<HashMap<String, BibEntry>, String> {
    let mut entries = HashMap::new();
    let chars: Vec<char> = bib_content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '@' {
            i += 1;
            let (entry_type, new_i) = parse_identifier(&chars, i);
            if entry_type.is_empty() {
                i = new_i;
                continue;
            }
            i = new_i;
            i = skip_whitespace(&chars, i);
            if i < len && chars[i] == '{' {
                i += 1;
                i = skip_whitespace(&chars, i);
                let (key, new_i) = parse_entry_key(&chars, i);
                if key.is_empty() {
                    i = find_matching_brace(&chars, new_i);
                    if i < len {
                        i += 1;
                    }
                    continue;
                }
                i = new_i;
                i = skip_whitespace(&chars, i);
                if i < len && chars[i] == ',' {
                    i += 1;
                }
                i = skip_whitespace(&chars, i);

                let (fields, new_i) = parse_fields(&chars, i);
                i = new_i;

                entries.insert(
                    key.clone(),
                    BibEntry {
                        entry_type: entry_type.to_lowercase(),
                        key,
                        fields,
                    },
                );
            }
        } else {
            i += 1;
        }
    }

    Ok(entries)
}

fn skip_whitespace(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    i
}

fn parse_identifier(chars: &[char], start: usize) -> (String, usize) {
    let mut end = start;
    while end < chars.len() && (chars[end].is_alphabetic() || chars[end] == '_') {
        end += 1;
    }
    let s: String = chars[start..end].iter().collect();
    (s, end)
}

fn parse_entry_key(chars: &[char], start: usize) -> (String, usize) {
    let mut end = start;
    while end < chars.len() && !chars[end].is_whitespace() && chars[end] != ',' && chars[end] != '}'
    {
        end += 1;
    }
    let key: String = chars[start..end].iter().collect();
    (key.trim().to_string(), end)
}

fn parse_fields(chars: &[char], start: usize) -> (HashMap<String, String>, usize) {
    let mut fields = HashMap::new();
    let mut i = start;

    while i < chars.len() && chars[i] != '}' {
        i = skip_whitespace(chars, i);
        if i >= chars.len() || chars[i] == '}' {
            break;
        }

        let (name, new_i) = parse_identifier(chars, i);
        if name.is_empty() {
            if i < chars.len() && chars[i] == '}' {
                break;
            }
            i += 1;
            continue;
        }
        i = new_i;
        i = skip_whitespace(chars, i);

        if i >= chars.len() || chars[i] != '=' {
            continue;
        }
        i += 1;
        i = skip_whitespace(chars, i);

        if i >= chars.len() {
            break;
        }

        let (value, new_i) = if chars[i] == '{' {
            parse_braced_value(chars, i)
        } else if chars[i] == '"' {
            parse_quoted_value(chars, i)
        } else {
            parse_numeric_value(chars, i)
        };
        i = new_i;
        fields.insert(name.to_lowercase(), value);

        i = skip_whitespace(chars, i);
        if i < chars.len() && chars[i] == ',' {
            i += 1;
        }
    }

    if i < chars.len() && chars[i] == '}' {
        i += 1;
    }

    (fields, i)
}

fn parse_braced_value(chars: &[char], start: usize) -> (String, usize) {
    if start >= chars.len() || chars[start] != '{' {
        return (String::new(), start);
    }
    let mut depth: usize = 1;
    let mut i = start + 1;
    let mut value = String::new();

    while i < chars.len() && depth > 0 {
        if chars[i] == '{' {
            depth += 1;
            value.push('{');
        } else if chars[i] == '}' {
            depth -= 1;
            if depth > 0 {
                value.push('}');
            }
        } else {
            value.push(chars[i]);
        }
        i += 1;
    }

    (value.trim().to_string(), i)
}

fn parse_quoted_value(chars: &[char], start: usize) -> (String, usize) {
    if start >= chars.len() || chars[start] != '"' {
        return (String::new(), start);
    }
    let mut i = start + 1;
    let mut value = String::new();

    while i < chars.len() && chars[i] != '"' {
        if chars[i] == '{' {
            let (inner, new_i) = parse_braced_value(chars, i);
            value.push_str(&inner);
            i = new_i;
        } else {
            value.push(chars[i]);
            i += 1;
        }
    }

    if i < chars.len() && chars[i] == '"' {
        i += 1;
    }

    (value.trim().to_string(), i)
}

fn parse_numeric_value(chars: &[char], start: usize) -> (String, usize) {
    let mut end = start;
    while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '-') {
        end += 1;
    }
    let value: String = chars[start..end].iter().collect();
    (value, end)
}

fn find_matching_brace(chars: &[char], start: usize) -> usize {
    let mut depth: usize = 0;
    let mut i = start;
    while i < chars.len() {
        if chars[i] == '{' {
            depth += 1;
        } else if chars[i] == '}' {
            if depth == 0 {
                return i;
            }
            depth -= 1;
        }
        i += 1;
    }
    i
}

pub fn format_citation_ieee(entry: &BibEntry) -> String {
    let author = entry
        .fields
        .get("author")
        .map(|s| s.as_str())
        .unwrap_or("Unknown");
    let title = entry
        .fields
        .get("title")
        .map(|s| s.as_str())
        .unwrap_or("Untitled");
    let year = entry
        .fields
        .get("year")
        .map(|s| s.as_str())
        .unwrap_or("n.d.");

    match entry.entry_type.as_str() {
        "article" => {
            let journal = entry
                .fields
                .get("journal")
                .map(|s| s.as_str())
                .unwrap_or("");
            let volume = entry.fields.get("volume").map(|s| s.as_str()).unwrap_or("");
            let pages = entry.fields.get("pages").map(|s| s.as_str()).unwrap_or("");
            if !journal.is_empty() && !volume.is_empty() && !pages.is_empty() {
                format!(
                    "{}, \"{},\" {}, vol. {}, pp. {}, {}.",
                    author, title, journal, volume, pages, year
                )
            } else if !journal.is_empty() {
                format!("{}, \"{},\" {}, {}.", author, title, journal, year)
            } else {
                format!("{}, \"{},\" {}.", author, title, year)
            }
        }
        "book" => {
            let publisher = entry
                .fields
                .get("publisher")
                .map(|s| s.as_str())
                .unwrap_or("");
            if !publisher.is_empty() {
                format!("{}, *{}*, {}. {}.", author, title, publisher, year)
            } else {
                format!("{}, *{}*, {}.", author, title, year)
            }
        }
        "inproceedings" | "conference" => {
            let booktitle = entry
                .fields
                .get("booktitle")
                .map(|s| s.as_str())
                .unwrap_or("");
            let pages = entry.fields.get("pages").map(|s| s.as_str()).unwrap_or("");
            if !booktitle.is_empty() && !pages.is_empty() {
                format!(
                    "{}, \"{},\" in *Proc. {}*, pp. {}, {}.",
                    author, title, booktitle, pages, year
                )
            } else if !booktitle.is_empty() {
                format!(
                    "{}, \"{},\" in *Proc. {}*, {}.",
                    author, title, booktitle, year
                )
            } else {
                format!("{}, \"{},\" {}.", author, title, year)
            }
        }
        _ => {
            format!("{}, \"{},\" {}.", author, title, year)
        }
    }
}

pub fn format_citation_apa(entry: &BibEntry) -> String {
    let author = entry
        .fields
        .get("author")
        .map(|s| s.as_str())
        .unwrap_or("Unknown");
    let title = entry
        .fields
        .get("title")
        .map(|s| s.as_str())
        .unwrap_or("Untitled");
    let year = entry
        .fields
        .get("year")
        .map(|s| s.as_str())
        .unwrap_or("n.d.");

    match entry.entry_type.as_str() {
        "article" => {
            let journal = entry
                .fields
                .get("journal")
                .map(|s| s.as_str())
                .unwrap_or("");
            let volume = entry.fields.get("volume").map(|s| s.as_str()).unwrap_or("");
            let pages = entry.fields.get("pages").map(|s| s.as_str()).unwrap_or("");
            if !journal.is_empty() && !volume.is_empty() && !pages.is_empty() {
                let pages_dash = pages.replace("--", "-");
                let vol_pages = format!("*{}, {}*", volume, pages_dash);
                format!(
                    "{} ({}). {}. {}, {}.",
                    author, year, title, journal, vol_pages
                )
            } else if !journal.is_empty() {
                format!("{} ({}). {}. {}.", author, year, title, journal)
            } else {
                format!("{} ({}). {}.", author, year, title)
            }
        }
        "book" => {
            format!("{} ({}). *{}*.", author, year, title)
        }
        "inproceedings" | "conference" => {
            let booktitle = entry
                .fields
                .get("booktitle")
                .map(|s| s.as_str())
                .unwrap_or("");
            if !booktitle.is_empty() {
                format!("{} ({}). {}. In *{}*.", author, year, title, booktitle)
            } else {
                format!("{} ({}). {}.", author, year, title)
            }
        }
        _ => {
            format!("{} ({}). {}.", author, year, title)
        }
    }
}

pub fn format_citation_chicago(entry: &BibEntry) -> String {
    let author = entry
        .fields
        .get("author")
        .map(|s| s.as_str())
        .unwrap_or("Unknown");
    let title = entry
        .fields
        .get("title")
        .map(|s| s.as_str())
        .unwrap_or("Untitled");
    let year = entry
        .fields
        .get("year")
        .map(|s| s.as_str())
        .unwrap_or("n.d.");

    match entry.entry_type.as_str() {
        "article" => {
            let journal = entry
                .fields
                .get("journal")
                .map(|s| s.as_str())
                .unwrap_or("");
            let volume = entry.fields.get("volume").map(|s| s.as_str()).unwrap_or("");
            let pages = entry.fields.get("pages").map(|s| s.as_str()).unwrap_or("");
            if !journal.is_empty() && !volume.is_empty() && !pages.is_empty() {
                let pages_dash = pages.replace("--", "-");
                format!(
                    "{}. \"{}.\" *{}* {} ({}): {}.",
                    author, title, journal, volume, year, pages_dash
                )
            } else if !journal.is_empty() && !volume.is_empty() {
                format!(
                    "{}. \"{}.\" *{}* {} ({}).",
                    author, title, journal, volume, year
                )
            } else if !journal.is_empty() {
                format!("{}. \"{}.\" *{}* ({}).", author, title, journal, year)
            } else {
                format!("{}. \"{}.\" {}.", author, title, year)
            }
        }
        "book" => {
            let publisher = entry
                .fields
                .get("publisher")
                .map(|s| s.as_str())
                .unwrap_or("");
            if !publisher.is_empty() {
                format!("{}. *{}*. {}, {}.", author, title, publisher, year)
            } else {
                format!("{}. *{}*. {}.", author, title, year)
            }
        }
        "inproceedings" | "conference" => {
            let booktitle = entry
                .fields
                .get("booktitle")
                .map(|s| s.as_str())
                .unwrap_or("");
            let pages = entry.fields.get("pages").map(|s| s.as_str()).unwrap_or("");
            if !booktitle.is_empty() && !pages.is_empty() {
                let pages_dash = pages.replace("--", "-");
                format!(
                    "{}. \"{}.\" In *{}, {}*. {}.",
                    author, title, booktitle, pages_dash, year
                )
            } else if !booktitle.is_empty() {
                format!("{}. \"{}.\" In *{}*. {}.", author, title, booktitle, year)
            } else {
                format!("{}. \"{}.\" {}.", author, title, year)
            }
        }
        _ => {
            format!("{}. \"{}.\" {}.", author, title, year)
        }
    }
}

pub fn disambiguate_year(entries: &[(String, String, String)]) -> HashMap<String, String> {
    let mut author_year_groups: HashMap<(String, String), Vec<&str>> = HashMap::new();
    for (key, author, year) in entries {
        let group = author_year_groups
            .entry((author.clone(), year.clone()))
            .or_default();
        group.push(key);
    }

    let mut map = HashMap::new();
    for (key, author, year) in entries {
        let group_key = (author.clone(), year.clone());
        if let Some(group) = author_year_groups.get(&group_key) {
            if group.len() <= 1 {
                map.insert(key.clone(), year.clone());
            } else {
                let idx = group.iter().position(|&k| k == key).unwrap_or(0);
                if idx == 0 {
                    map.insert(key.clone(), year.clone());
                } else {
                    let suffix = b'a' + (idx - 1) as u8;
                    map.insert(key.clone(), format!("{}{}", year, suffix as char));
                }
            }
        }
    }
    map
}

pub fn format_citation_with_disambiguation(
    style: &str,
    entry: &BibEntry,
    disambig_map: &HashMap<String, String>,
) -> String {
    let year = disambig_map.get(&entry.key).cloned().unwrap_or_else(|| {
        entry
            .fields
            .get("year")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "n.d.".to_string())
    });

    let mut entry_with_disambig = entry.clone();
    entry_with_disambig.fields.insert("year".to_string(), year);

    match style.to_lowercase().as_str() {
        "ieee" => format_citation_ieee(&entry_with_disambig),
        "apa" => format_citation_apa(&entry_with_disambig),
        "chicago" => format_citation_chicago(&entry_with_disambig),
        _ => format_citation_ieee(&entry_with_disambig),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_BIB: &str = r#"
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
    fn test_parse_simple_bib() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        assert_eq!(entries.len(), 4);
        assert!(entries.contains_key("knuth1984"));
        assert!(entries.contains_key("lamport1994"));
        assert!(entries.contains_key("dijkstra1968"));
        assert!(entries.contains_key("rust2024"));
    }

    #[test]
    fn test_parse_article_fields() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("knuth1984").expect("entry should exist");
        assert_eq!(entry.entry_type, "article");
        assert_eq!(entry.key, "knuth1984");
        assert_eq!(
            entry.fields.get("author").map(|s| s.as_str()),
            Some("Donald E. Knuth")
        );
        assert_eq!(
            entry.fields.get("title").map(|s| s.as_str()),
            Some("Literate Programming")
        );
        assert_eq!(entry.fields.get("volume").map(|s| s.as_str()), Some("27"));
        assert_eq!(
            entry.fields.get("pages").map(|s| s.as_str()),
            Some("97--111")
        );
    }

    #[test]
    fn test_parse_book_fields() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("lamport1994").expect("entry should exist");
        assert_eq!(entry.entry_type, "book");
        assert_eq!(
            entry.fields.get("publisher").map(|s| s.as_str()),
            Some("Addison-Wesley")
        );
    }

    #[test]
    fn test_parse_inproceedings() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("dijkstra1968").expect("entry should exist");
        assert_eq!(entry.entry_type, "inproceedings");
        assert_eq!(
            entry.fields.get("booktitle").map(|s| s.as_str()),
            Some("Proc. IFIP Congress")
        );
    }

    #[test]
    fn test_parse_quoted_fields() {
        let bib = r#"@article{key1, author = "Jane Doe", title = "A Title", year = "2020"}"#;
        let entries = parse_bib(bib).expect("parse should succeed");
        let entry = entries.get("key1").expect("entry should exist");
        assert_eq!(
            entry.fields.get("author").map(|s| s.as_str()),
            Some("Jane Doe")
        );
        assert_eq!(entry.fields.get("year").map(|s| s.as_str()), Some("2020"));
    }

    #[test]
    fn test_parse_numeric_year() {
        let bib = r#"@article{key2, author = {Someone}, title = {Something}, year = 2021}"#;
        let entries = parse_bib(bib).expect("parse should succeed");
        let entry = entries.get("key2").expect("entry should exist");
        assert_eq!(entry.fields.get("year").map(|s| s.as_str()), Some("2021"));
    }

    #[test]
    fn test_parse_empty_bib() {
        let entries = parse_bib("").expect("parse should succeed");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_nested_braces() {
        let bib = r#"@book{key3, author = {Some {Nested} Name}, title = {A {Book} Title}, year = {2022}}"#;
        let entries = parse_bib(bib).expect("parse should succeed");
        let entry = entries.get("key3").expect("entry should exist");
        assert_eq!(
            entry.fields.get("author").map(|s| s.as_str()),
            Some("Some {Nested} Name")
        );
    }

    #[test]
    fn test_format_citation_ieee_article() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("knuth1984").expect("entry should exist");
        let formatted = format_citation_ieee(entry);
        assert!(formatted.contains("Donald E. Knuth"));
        assert!(formatted.contains("Literate Programming"));
        assert!(formatted.contains("1984"));
        assert!(formatted.contains("The Computer Journal"));
    }

    #[test]
    fn test_format_citation_ieee_book() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("lamport1994").expect("entry should exist");
        let formatted = format_citation_ieee(entry);
        assert!(formatted.contains("Leslie Lamport"));
        assert!(formatted.contains("Addison-Wesley"));
        assert!(formatted.contains("1994"));
    }

    #[test]
    fn test_format_citation_ieee_inproceedings() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("dijkstra1968").expect("entry should exist");
        let formatted = format_citation_ieee(entry);
        assert!(formatted.contains("Edsger W. Dijkstra"));
        assert!(formatted.contains("Go To Statement Considered Harmful"));
        assert!(formatted.contains("Proc. IFIP Congress"));
    }

    #[test]
    fn test_format_citation_apa_article() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("knuth1984").expect("entry should exist");
        let formatted = format_citation_apa(entry);
        assert!(formatted.contains("Donald E. Knuth"));
        assert!(formatted.contains("1984"));
        assert!(formatted.starts_with("Donald E. Knuth (1984)."));
    }

    #[test]
    fn test_format_citation_apa_book() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("lamport1994").expect("entry should exist");
        let formatted = format_citation_apa(entry);
        assert!(formatted.contains("Leslie Lamport (1994)."));
    }

    #[test]
    fn test_format_citation_apa_inproceedings() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("dijkstra1968").expect("entry should exist");
        let formatted = format_citation_apa(entry);
        assert!(formatted.contains("Edsger W. Dijkstra (1968)."));
        assert!(formatted.contains("In *Proc. IFIP Congress*."));
    }

    #[test]
    fn test_fields_are_lowercase() {
        let bib = r#"@article{key4, AUTHOR = "Upper", TITLE = "Upper Title", YEAR = "2023"}"#;
        let entries = parse_bib(bib).expect("parse should succeed");
        let entry = entries.get("key4").expect("entry should exist");
        assert!(entry.fields.contains_key("author"));
        assert!(entry.fields.contains_key("title"));
        assert!(entry.fields.contains_key("year"));
    }

    #[test]
    fn test_format_citation_chicago_article() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("knuth1984").expect("entry should exist");
        let formatted = format_citation_chicago(entry);
        assert!(formatted.contains("Donald E. Knuth"));
        assert!(formatted.contains("Literate Programming"));
        assert!(formatted.contains("The Computer Journal"));
        assert!(formatted.contains("27"));
        assert!(formatted.contains("1984"));
        assert!(formatted.contains("97-111"));
    }

    #[test]
    fn test_format_citation_chicago_book() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("lamport1994").expect("entry should exist");
        let formatted = format_citation_chicago(entry);
        assert!(formatted.contains("Leslie Lamport"));
        assert!(formatted.contains("Addison-Wesley"));
        assert!(formatted.contains("1994"));
    }

    #[test]
    fn test_format_citation_chicago_inproceedings() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("dijkstra1968").expect("entry should exist");
        let formatted = format_citation_chicago(entry);
        assert!(formatted.contains("Edsger W. Dijkstra"));
        assert!(formatted.contains("Go To Statement Considered Harmful"));
        assert!(formatted.contains("Proc. IFIP Congress"));
        assert!(formatted.contains("1-5"));
    }

    #[test]
    fn test_format_citation_chicago_misc() {
        let entries = parse_bib(SIMPLE_BIB).expect("parse should succeed");
        let entry = entries.get("rust2024").expect("entry should exist");
        let formatted = format_citation_chicago(entry);
        assert!(formatted.contains("The Rust Project Developers"));
        assert!(formatted.contains("The Rust Programming Language"));
        assert!(formatted.contains("2024"));
    }

    #[test]
    fn test_disambiguate_year_unique() {
        let entries = vec![
            ("key1".to_string(), "Alice".to_string(), "2024".to_string()),
            ("key2".to_string(), "Bob".to_string(), "2023".to_string()),
        ];
        let map = disambiguate_year(&entries);
        assert_eq!(map.get("key1").map(|s| s.as_str()), Some("2024"));
        assert_eq!(map.get("key2").map(|s| s.as_str()), Some("2023"));
    }

    #[test]
    fn test_disambiguate_year_same_author_year() {
        let entries = vec![
            (
                "smith2024a".to_string(),
                "Jane Smith".to_string(),
                "2024".to_string(),
            ),
            (
                "smith2024b".to_string(),
                "Jane Smith".to_string(),
                "2024".to_string(),
            ),
        ];
        let map = disambiguate_year(&entries);
        assert_eq!(map.get("smith2024a").map(|s| s.as_str()), Some("2024"));
        assert_eq!(map.get("smith2024b").map(|s| s.as_str()), Some("2024a"));
    }

    #[test]
    fn test_disambiguate_year_three_same() {
        let entries = vec![
            ("k1".to_string(), "A".to_string(), "2020".to_string()),
            ("k2".to_string(), "A".to_string(), "2020".to_string()),
            ("k3".to_string(), "A".to_string(), "2020".to_string()),
        ];
        let map = disambiguate_year(&entries);
        assert_eq!(map.get("k1").map(|s| s.as_str()), Some("2020"));
        assert_eq!(map.get("k2").map(|s| s.as_str()), Some("2020a"));
        assert_eq!(map.get("k3").map(|s| s.as_str()), Some("2020b"));
    }

    #[test]
    fn test_format_with_disambiguation_chicago() {
        let entries = vec![
            (
                "smith2024a".to_string(),
                "Jane Smith".to_string(),
                "2024".to_string(),
            ),
            (
                "smith2024b".to_string(),
                "Jane Smith".to_string(),
                "2024".to_string(),
            ),
        ];
        let map = disambiguate_year(&entries);

        let bib = r#"@article{smith2024a, author = {Jane Smith}, title = {First Paper}, journal = {J. Examples}, volume = {1}, pages = {1--10}, year = {2024}}"#;
        let parsed = parse_bib(bib).expect("parse should succeed");
        let entry = parsed.get("smith2024a").expect("entry should exist");
        let formatted = format_citation_with_disambiguation("chicago", entry, &map);
        assert!(formatted.contains("2024"));
        assert!(!formatted.contains("2024a"));

        let bib2 = r#"@article{smith2024b, author = {Jane Smith}, title = {Second Paper}, journal = {J. Examples}, volume = {1}, pages = {11--20}, year = {2024}}"#;
        let parsed2 = parse_bib(bib2).expect("parse should succeed");
        let entry2 = parsed2.get("smith2024b").expect("entry should exist");
        let formatted2 = format_citation_with_disambiguation("chicago", entry2, &map);
        assert!(formatted2.contains("2024a"));
    }

    #[test]
    fn test_format_with_disambiguation_unknown_style() {
        let bib = r#"@article{testkey, author = {A. Author}, title = {Test}, journal = {J. Test}, volume = {1}, pages = {1--5}, year = {2020}}"#;
        let parsed = parse_bib(bib).expect("parse should succeed");
        let entry = parsed.get("testkey").expect("entry should exist");
        let map = HashMap::new();
        let formatted = format_citation_with_disambiguation("unknown", entry, &map);
        assert!(formatted.contains("A. Author"));
        assert!(formatted.contains("2020"));
    }
}
