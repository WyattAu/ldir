//! S-IR v2 Module — the top-level container.

use crate::sir::v2::annotations::{Annotations, LabelCategory};
use crate::sir::v2::metadata::DocumentMetadata;
use crate::sir::v2::nodes::{Node, NodeTree};
use crate::sir::v2::resources::ResourceDecls;
use crate::sir::v2::styles::StyleDecls;

/// S-IR format version.
pub const SIR_V2_VERSION: (u8, u8, u8) = (2, 0, 0);
pub const SIR_V2_MAGIC: &[u8; 4] = b"LDIR";

/// Module header for S-IR v2 binary format.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleHeader {
    pub magic: [u8; 4],
    pub version: (u8, u8, u8),
    pub ir_version: u16,
    pub source_format: Option<String>,
    pub source_path: Option<String>,
    pub created: u64,
}

impl Default for ModuleHeader {
    fn default() -> Self {
        Self {
            magic: *SIR_V2_MAGIC,
            version: SIR_V2_VERSION,
            ir_version: 2,
            source_format: None,
            source_path: None,
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

/// S-IR v2 Module — the complete, self-contained document representation.
///
/// This is the top-level container for a document in the ldir IR.
/// It can be serialized to binary or text format, and deserialized back.
///
/// # Structure
///
/// ```text
/// SIRModuleV2 {
///     header: ModuleHeader,
///     metadata: DocumentMetadata,
///     resources: ResourceDecls,
///     styles: StyleDecls,
///     annotations: Annotations,
///     body: NodeTree,
/// }
/// ```
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[doc(alias = "SIRModule")]
#[doc(alias = "Module")]
pub struct SIRModuleV2 {
    pub header: ModuleHeader,
    pub metadata: DocumentMetadata,
    pub resources: ResourceDecls,
    pub styles: StyleDecls,
    pub annotations: Annotations,
    pub body: NodeTree,
}

impl SIRModuleV2 {
    /// Create a new empty module.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new module with source tracking.
    pub fn from_source(format: &str, path: &str) -> Self {
        let mut m = Self::new();
        m.header.source_format = Some(format.to_string());
        m.header.source_path = Some(path.to_string());
        m
    }

    /// Add a labeled node and register its label.
    pub fn add_labeled_node(
        &mut self,
        mut node: Node,
        label: &str,
        category: LabelCategory,
    ) -> u32 {
        let id = node.id;
        node.label = Some(label.to_string());
        self.annotations.add_label(label.to_string(), id, category);
        self.body.push(node)
    }

    /// Collect all heading nodes in document order.
    pub fn headings(&self) -> Vec<&Node> {
        self.body.find_by_type(|nt| {
            matches!(
                nt,
                crate::sir::v2::nodes::NodeType::Part
                    | crate::sir::v2::nodes::NodeType::Chapter
                    | crate::sir::v2::nodes::NodeType::Section
                    | crate::sir::v2::nodes::NodeType::Subsection
                    | crate::sir::v2::nodes::NodeType::Subsubsection
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sir::v2::nodes::NodeType;

    #[test]
    fn test_new_module() {
        let m = SIRModuleV2::new();
        assert_eq!(m.header.version, (2, 0, 0));
        assert_eq!(m.header.magic, *b"LDIR");
        assert!(m.metadata.title.is_none());
        assert!(m.body.is_empty());
    }

    #[test]
    fn test_from_source() {
        let m = SIRModuleV2::from_source("latex", "main.tex");
        assert_eq!(m.header.source_format.as_deref(), Some("latex"));
        assert_eq!(m.header.source_path.as_deref(), Some("main.tex"));
    }

    #[test]
    fn test_add_labeled_node() {
        let mut m = SIRModuleV2::new();
        let id = m.add_labeled_node(
            Node::new(1, NodeType::Section),
            "sec:intro",
            LabelCategory::Section,
        );
        assert_eq!(id, 1);
        assert!(m.annotations.find_label("sec:intro").is_some());
        assert_eq!(
            m.annotations.find_label("sec:intro").unwrap().category,
            LabelCategory::Section
        );
        assert_eq!(m.body.find_by_label("sec:intro").unwrap().id, 1);
    }

    #[test]
    fn test_headings() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Chapter));
        m.body.push(Node::new(2, NodeType::Section));
        m.body.push(Node::new(3, NodeType::Paragraph));
        m.body.push(Node::new(4, NodeType::Subsection));

        let h = m.headings();
        assert_eq!(h.len(), 3);
    }
}
