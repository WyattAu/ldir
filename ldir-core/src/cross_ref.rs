//! Cross-reference resolution for document labels and citations.

/// A resolved reference target with its location and context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRef {
    /// Label or identifier being referenced.
    pub label: String,
    /// Page number where the target is found (1-indexed, 0 if unknown).
    pub page: u32,
    /// Section number if heading (e.g., "2.3.1").
    pub section: Option<String>,
    /// Reference type: internal link, external URL, bibliography.
    pub ref_type: RefType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The kind of target a reference points at.
pub enum RefType {
    /// A reference to a label within the same document.
    Internal,
    /// A reference to an external resource (URL).
    External,
    /// A citation into the bibliography.
    Bibliography,
    /// A numbered equation.
    Equation,
    /// A numbered figure.
    Figure,
    /// A numbered table.
    Table,
}

/// A collection of all label definitions in a document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LabelRegistry {
    entries: Vec<LabelEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A single registered label definition.
pub struct LabelEntry {
    /// The label name.
    pub label: String,
    /// Page where the target appears (1-indexed).
    pub page: u32,
    /// Section number if the target is a heading.
    pub section: Option<String>,
    /// The S-IR node type of the target.
    pub node_type: String,
}

impl LabelRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a label definition.
    pub fn register(&mut self, label: String, page: u32, section: Option<String>, node_type: &str) {
        if !self.entries.iter().any(|e| e.label == label) {
            self.entries.push(LabelEntry {
                label,
                page,
                section,
                node_type: node_type.to_string(),
            });
        }
    }

    /// Look up a label by name.
    pub fn lookup(&self, label: &str) -> Option<&LabelEntry> {
        self.entries.iter().find(|e| e.label == label)
    }

    /// Check if all labels referenced in refs have definitions.
    pub fn unresolved_refs<'a>(&self, refs: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
        refs.filter(|r| self.lookup(r).is_none()).collect()
    }

    /// Return count of registered labels.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Resolve a reference string (e.g., "\\ref{fig:sunset}") to a resolved ref.
pub fn resolve_ref(raw: &str) -> ResolvedRef {
    let label = raw.trim_start_matches("\\ref{").trim_end_matches('}');
    let (label, ref_type) = if label.ends_with(":url") {
        (label.trim_end_matches(":url"), RefType::External)
    } else if label.ends_with(":bib") {
        (label.trim_end_matches(":bib"), RefType::Bibliography)
    } else if label.starts_with("eq:") {
        (label, RefType::Equation)
    } else if label.starts_with("fig:") {
        (label, RefType::Figure)
    } else if label.starts_with("tbl:") {
        (label, RefType::Table)
    } else {
        (label, RefType::Internal)
    };

    ResolvedRef {
        label: label.to_string(),
        page: 0,
        section: None,
        ref_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut reg = LabelRegistry::new();
        reg.register("fig:sunset".into(), 1, None, "Figure");
        reg.register("eq:euler".into(), 2, Some("2.1".into()), "Equation");
        assert!(reg.lookup("fig:sunset").is_some());
        assert!(reg.lookup("eq:euler").is_some());
        assert!(reg.lookup("missing").is_none());
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn test_no_duplicate_registration() {
        let mut reg = LabelRegistry::new();
        reg.register("intro".into(), 1, None, "Heading");
        reg.register("intro".into(), 2, None, "Heading");
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_unresolved_refs_detection() {
        let mut reg = LabelRegistry::new();
        reg.register("exists".into(), 1, None, "Paragraph");
        let refs = ["missing", "exists", "also_missing"];
        let unresolved = reg.unresolved_refs(refs.iter().copied());
        assert_eq!(unresolved.len(), 2);
    }

    #[test]
    fn test_resolve_internal_ref() {
        let r = resolve_ref("\\ref{sec:intro}");
        assert_eq!(r.label, "sec:intro");
        assert_eq!(r.ref_type, RefType::Internal);
    }

    #[test]
    fn test_resolve_external_ref() {
        let r = resolve_ref("\\ref{https://example.com:url}");
        assert_eq!(r.ref_type, RefType::External);
    }

    #[test]
    fn test_resolve_bib_ref() {
        let r = resolve_ref("\\ref{key:2024:bib}");
        assert_eq!(r.ref_type, RefType::Bibliography);
    }

    #[test]
    fn test_resolve_eq_ref() {
        let r = resolve_ref("\\ref{eq:pythagoras}");
        assert_eq!(r.ref_type, RefType::Equation);
    }

    #[test]
    fn test_resolve_fig_ref() {
        let r = resolve_ref("\\ref{fig:results}");
        assert_eq!(r.ref_type, RefType::Figure);
    }

    #[test]
    fn test_resolve_tbl_ref() {
        let r = resolve_ref("\\ref{tbl:datapoints}");
        assert_eq!(r.ref_type, RefType::Table);
    }

    #[test]
    fn test_resolve_plain_ref() {
        let r = resolve_ref("\\ref{summary}");
        assert_eq!(r.label, "summary");
        assert_eq!(r.ref_type, RefType::Internal);
    }

    #[test]
    fn test_registry_empty() {
        let reg = LabelRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_registry_unresolved_with_empty_refs() {
        let reg = LabelRegistry::new();
        let refs: Vec<&str> = vec![];
        let unresolved = reg.unresolved_refs(refs.into_iter());
        assert!(unresolved.is_empty());
    }
}
