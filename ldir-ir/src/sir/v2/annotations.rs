//! Cross-reference annotations for S-IR v2.

use serde::{Deserialize, Serialize};

/// Category of a labeled entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LabelCategory {
    Section,
    Equation,
    Figure,
    Table,
    Footnote,
    Page,
    Custom,
}

/// Information about a label in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelInfo {
    pub node_id: u32,
    pub category: LabelCategory,
}

/// A cross-reference to a label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRef {
    pub label: String,
    pub ref_node_id: u32, // node containing the \ref
}

/// Annotations collection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Annotations {
    pub labels: std::collections::HashMap<String, LabelInfo>,
    pub refs: Vec<CrossRef>,
}

impl Annotations {
    pub fn add_label(&mut self, label: String, node_id: u32, category: LabelCategory) {
        self.labels.insert(label, LabelInfo { node_id, category });
    }

    pub fn add_ref(&mut self, label: String, ref_node_id: u32) {
        self.refs.push(CrossRef { label, ref_node_id });
    }

    pub fn find_label(&self, label: &str) -> Option<&LabelInfo> {
        self.labels.get(label)
    }
}
