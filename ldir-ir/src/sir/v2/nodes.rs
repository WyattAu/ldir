//! Document node types for S-IR v2.
//!
//! The document body is a tree of typed nodes. Each node has a unique ID,
//! a type, type-specific properties, optional children, and optional annotations.

use serde::{Deserialize, Serialize};

use crate::sir::v2::metadata::Dimension;

/// Node type enumeration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    // === Document structure ===
    Document,
    Part,
    Chapter,
    Section,
    Subsection,
    Subsubsection,
    Paragraph,
    
    // === Lists ===
    List { list_type: ListType, ordered: bool, start: Option<u32> },
    ListItem,
    
    // === Block content ===
    BlockQuote,
    CodeBlock { language: Option<String> },
    MathBlock { math_type: MathType, numbered: bool },
    Table { col_specs: Vec<ColSpec>, num_cols: usize },
    TableRow { is_header: bool },
    TableCell { colspan: u8, rowspan: u8 },
    
    // === Inline content (can be children of any block) ===
    Text { content: String },
    Styled { style_name: String },
    Bold,
    Italic,
    Mono,
    Underline,
    Strikethrough,
    SmallCaps,
    Link { url: String, title: Option<String> },
    Image { source: String, alt: String, width: Option<Dimension>, height: Option<Dimension> },
    MathInline { content: String },
    LineBreak,
    
    // === Floats ===
    Figure { placement: FloatPlacement },
    Caption,
    
    // === Special ===
    Footnote { content: String },
    FootnoteBlock,
    TableOfContents { max_depth: u8 },
    PageBreak,
    ThematicBreak,
    Citation { keys: Vec<String>, style: Option<String> },
    
    // === Container ===
    Group,  // anonymous grouping node
}

/// List type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ListType {
    Unordered,
    Ordered,
    Description,
}

/// Math block type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MathType {
    Equation,
    Align,
    Gather,
    Multline,
    Cases,
    Matrix { delimiters: (Option<char>, Option<char>) },
}

/// Float placement hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FloatPlacement {
    Here,
    Top,
    Bottom,
    Page,
    ForceHere,
}

/// Table column specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColSpec {
    pub align: ColumnAlign,
    pub width: Option<Dimension>,
}

/// Column alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColumnAlign {
    Left,
    Right,
    Center,
    Justified,
}

/// A single document node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u32,
    pub node_type: NodeType,
    pub parent_id: Option<u32>,
    pub child_ids: Vec<u32>,
    pub label: Option<String>,
    pub style: Option<String>,
    pub counter: Option<String>,
}

impl Node {
    pub fn new(id: u32, node_type: NodeType) -> Self {
        Self {
            id,
            node_type,
            parent_id: None,
            child_ids: Vec::new(),
            label: None,
            style: None,
            counter: None,
        }
    }

    pub fn with_parent(mut self, parent_id: u32) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    pub fn with_counter(mut self, counter: impl Into<String>) -> Self {
        self.counter = Some(counter.into());
        self
    }

    pub fn add_child(&mut self, child_id: u32) {
        self.child_ids.push(child_id);
    }

    /// Get text content if this is a Text node.
    pub fn text_content(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Text { content } => Some(content),
            NodeType::Section | NodeType::Subsection | NodeType::Subsubsection
            | NodeType::Chapter | NodeType::Part => None,
            _ => None,
        }
    }

    /// Check if this node is a structural heading.
    pub fn is_heading(&self) -> bool {
        matches!(
            self.node_type,
            NodeType::Part | NodeType::Chapter | NodeType::Section
            | NodeType::Subsection | NodeType::Subsubsection
        )
    }

    /// Get heading level (0=part, 1=chapter, 2=section, etc.)
    pub fn heading_level(&self) -> Option<u8> {
        match &self.node_type {
            NodeType::Part => Some(0),
            NodeType::Chapter => Some(1),
            NodeType::Section => Some(2),
            NodeType::Subsection => Some(3),
            NodeType::Subsubsection => Some(4),
            _ => None,
        }
    }
}

/// The document body — a collection of nodes forming a tree.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeTree {
    nodes: Vec<Node>,
    root_ids: Vec<u32>,
}

impl NodeTree {
    pub fn new() -> Self { Self::default() }

    pub fn push(&mut self, node: Node) -> u32 {
        let id = node.id;
        if node.parent_id.is_none() {
            self.root_ids.push(id);
        }
        self.nodes.push(node);
        id
    }

    pub fn get(&self, id: u32) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn is_empty(&self) -> bool { self.nodes.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = &Node> { self.nodes.iter() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Node> { self.nodes.iter_mut() }
    pub fn roots(&self) -> &[u32] { &self.root_ids }

    pub fn retain(&mut self, f: impl FnMut(&Node) -> bool) {
        use std::collections::HashSet;
        let before: HashSet<u32> = self.nodes.iter().map(|n| n.id).collect();
        self.nodes.retain(f);
        let after: HashSet<u32> = self.nodes.iter().map(|n| n.id).collect();
        let removed: HashSet<u32> = before.difference(&after).copied().collect();
        for node in &mut self.nodes {
            node.child_ids.retain(|id| !removed.contains(id));
        }
        self.root_ids.retain(|id| !removed.contains(id));
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root_ids.clear();
    }

    pub fn rebuild_roots(&mut self) {
        self.root_ids = self.nodes.iter()
            .filter(|n| n.parent_id.is_none())
            .map(|n| n.id)
            .collect();
    }

    /// Find a node by label.
    pub fn find_by_label(&self, label: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.label.as_deref() == Some(label))
    }

    /// Get all nodes of a specific type.
    pub fn find_by_type<F>(&self, pred: F) -> Vec<&Node>
    where
        F: Fn(&NodeType) -> bool,
    {
        self.nodes.iter().filter(|n| pred(&n.node_type)).collect()
    }

    /// Collect text content from a node and its descendants.
    pub fn collect_text(&self, node_id: u32) -> String {
        let mut text = String::new();
        self.collect_text_recursive(node_id, &mut text);
        text
    }

    fn collect_text_recursive(&self, node_id: u32, text: &mut String) {
        if let Some(node) = self.get(node_id) {
            if let NodeType::Text { content } = &node.node_type {
                text.push_str(content);
            }
            for &child_id in &node.child_ids {
                self.collect_text_recursive(child_id, text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(1, NodeType::Document);
        assert_eq!(node.id, 1);
        assert!(node.parent_id.is_none());
        assert!(node.child_ids.is_empty());
        assert!(node.label.is_none());
    }

    #[test]
    fn test_node_builder_pattern() {
        let node = Node::new(2, NodeType::Section)
            .with_parent(1)
            .with_label("intro")
            .with_style("heading")
            .with_counter("section");

        assert_eq!(node.id, 2);
        assert_eq!(node.parent_id, Some(1));
        assert_eq!(node.label.as_deref(), Some("intro"));
        assert_eq!(node.style.as_deref(), Some("heading"));
        assert_eq!(node.counter.as_deref(), Some("section"));
    }

    #[test]
    fn test_node_tree_push_get() {
        let mut tree = NodeTree::new();
        tree.push(Node::new(1, NodeType::Document));
        tree.push(Node::new(2, NodeType::Section).with_parent(1));

        assert_eq!(tree.len(), 2);
        assert!(tree.get(1).is_some());
        assert!(tree.get(2).is_some());
        assert!(tree.get(99).is_none());
        assert_eq!(tree.roots().len(), 1);
        assert_eq!(tree.roots()[0], 1);
    }

    #[test]
    fn test_collect_text() {
        let mut tree = NodeTree::new();
        tree.push(Node::new(1, NodeType::Paragraph));
        tree.push(Node::new(2, NodeType::Text { content: "Hello ".to_string() }).with_parent(1));
        tree.push(Node::new(3, NodeType::Bold).with_parent(1));
        tree.push(Node::new(4, NodeType::Text { content: "world".to_string() }).with_parent(3));

        if let Some(p) = tree.get_mut(1) {
            p.add_child(2);
            p.add_child(3);
        }
        if let Some(b) = tree.get_mut(3) {
            b.add_child(4);
        }

        assert_eq!(tree.collect_text(1), "Hello world");
    }

    #[test]
    fn test_find_by_label() {
        let mut tree = NodeTree::new();
        tree.push(Node::new(1, NodeType::Section).with_label("sec:intro"));
        tree.push(Node::new(2, NodeType::Paragraph));

        assert!(tree.find_by_label("sec:intro").is_some());
        assert!(tree.find_by_label("nonexistent").is_none());
        assert_eq!(tree.find_by_label("sec:intro").unwrap().id, 1);
    }

    #[test]
    fn test_retain() {
        let mut tree = NodeTree::new();
        tree.push(Node::new(1, NodeType::Document));
        tree.push(Node::new(2, NodeType::Paragraph).with_parent(1));
        tree.push(Node::new(3, NodeType::Text { content: "Hello".to_string() }).with_parent(1));
        if let Some(d) = tree.get_mut(1) { d.add_child(2); d.add_child(3); }

        tree.retain(|n| n.id != 3);
        assert_eq!(tree.len(), 2);
        assert!(tree.get(3).is_none());
        if let Some(d) = tree.get(1) {
            assert_eq!(d.child_ids.len(), 1);
            assert_eq!(d.child_ids[0], 2);
        }
    }

    #[test]
    fn test_clear() {
        let mut tree = NodeTree::new();
        tree.push(Node::new(1, NodeType::Document));
        assert_eq!(tree.len(), 1);
        tree.clear();
        assert!(tree.is_empty());
    }

    #[test]
    fn test_rebuild_roots() {
        let mut tree = NodeTree::new();
        tree.push(Node::new(1, NodeType::Document));
        tree.push(Node::new(2, NodeType::Section).with_parent(1));
        assert_eq!(tree.roots().len(), 1);
        if let Some(s) = tree.get_mut(2) { s.parent_id = None; }
        tree.rebuild_roots();
        assert_eq!(tree.roots().len(), 2);
    }

    #[test]
    fn test_find_by_type() {
        let mut tree = NodeTree::new();
        tree.push(Node::new(1, NodeType::Section));
        tree.push(Node::new(2, NodeType::Paragraph));
        tree.push(Node::new(3, NodeType::Paragraph));

        let paragraphs = tree.find_by_type(|nt| matches!(nt, NodeType::Paragraph));
        assert_eq!(paragraphs.len(), 2);
    }

    #[test]
    fn test_heading_level() {
        let part = Node::new(0, NodeType::Part);
        let chapter = Node::new(1, NodeType::Chapter);
        let section = Node::new(2, NodeType::Section);
        let sub = Node::new(3, NodeType::Subsection);
        let subsub = Node::new(4, NodeType::Subsubsection);
        let para = Node::new(5, NodeType::Paragraph);

        assert_eq!(part.heading_level(), Some(0));
        assert_eq!(chapter.heading_level(), Some(1));
        assert_eq!(section.heading_level(), Some(2));
        assert_eq!(sub.heading_level(), Some(3));
        assert_eq!(subsub.heading_level(), Some(4));
        assert_eq!(para.heading_level(), None);

        assert!(part.is_heading());
        assert!(section.is_heading());
        assert!(!para.is_heading());
    }
}
