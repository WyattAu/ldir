#![deny(unsafe_code)]

use pdf_writer::types::StructRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureType {
    Document,
    Part,
    Chapter,
    Section,
    Subsection,
    Heading,
    Paragraph,
    List,
    ListItem,
    Table,
    TableRow,
    TableCell,
    Figure,
    Caption,
    CodeBlock,
    BlockQuote,
    MathBlock,
    Footnote,
    FootnoteBlock,
    TOC,
    ThematicBreak,
}

impl StructureType {
    pub fn to_struct_role(self) -> StructRole {
        match self {
            Self::Document => StructRole::Document,
            Self::Part => StructRole::Part,
            Self::Chapter => StructRole::Sect,
            Self::Section => StructRole::Sect,
            Self::Subsection => StructRole::Sect,
            Self::Heading => StructRole::H1,
            Self::Paragraph => StructRole::P,
            Self::List => StructRole::L,
            Self::ListItem => StructRole::LI,
            Self::Table => StructRole::Table,
            Self::TableRow => StructRole::TR,
            Self::TableCell => StructRole::TD,
            Self::Figure => StructRole::Figure,
            Self::Caption => StructRole::Caption,
            Self::CodeBlock => StructRole::Code,
            Self::BlockQuote => StructRole::BlockQuote,
            Self::MathBlock => StructRole::Formula,
            Self::Footnote => StructRole::Note,
            Self::FootnoteBlock => StructRole::Note,
            Self::TOC => StructRole::TOC,
            Self::ThematicBreak => StructRole::NonStruct,
        }
    }

    pub fn custom_role_name(self) -> Option<&'static [u8]> {
        match self {
            Self::Chapter => Some(b"Chapter"),
            Self::Subsection => Some(b"Subsection"),
            Self::Heading => Some(b"Heading"),
            Self::CodeBlock => Some(b"CodeBlock"),
            Self::MathBlock => Some(b"MathBlock"),
            Self::FootnoteBlock => Some(b"FootnoteBlock"),
            Self::ThematicBreak => Some(b"ThematicBreak"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructureNode {
    pub element_type: StructureType,
    pub children: Vec<StructureNode>,
    pub alt_text: Option<String>,
    pub page: u32,
    pub mcid: u32,
}

impl StructureNode {
    pub fn new(element_type: StructureType, page: u32, mcid: u32) -> Self {
        Self {
            element_type,
            children: Vec::new(),
            alt_text: None,
            page,
            mcid,
        }
    }

    pub fn with_children(element_type: StructureType, children: Vec<StructureNode>) -> Self {
        Self {
            element_type,
            children,
            alt_text: None,
            page: 0,
            mcid: 0,
        }
    }

    pub fn with_alt_text(mut self, text: impl Into<String>) -> Self {
        self.alt_text = Some(text.into());
        self
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structure_tree_creation() {
        let doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                StructureNode::with_children(
                    StructureType::Heading,
                    vec![StructureNode::new(StructureType::Paragraph, 1, 0)],
                ),
                StructureNode::new(StructureType::Paragraph, 1, 1),
            ],
        );
        assert_eq!(doc.element_type, StructureType::Document);
        assert_eq!(doc.children.len(), 2);
        assert_eq!(doc.children[1].element_type, StructureType::Paragraph);
    }

    #[test]
    fn structure_type_from_node_type() {
        assert_eq!(StructureType::Document.to_struct_role(), StructRole::Document);
        assert_eq!(StructureType::Paragraph.to_struct_role(), StructRole::P);
        assert_eq!(StructureType::Chapter.to_struct_role(), StructRole::Sect);
        assert_eq!(StructureType::List.to_struct_role(), StructRole::L);
        assert_eq!(StructureType::ListItem.to_struct_role(), StructRole::LI);
        assert_eq!(StructureType::Table.to_struct_role(), StructRole::Table);
        assert_eq!(StructureType::Figure.to_struct_role(), StructRole::Figure);
        assert_eq!(StructureType::Caption.to_struct_role(), StructRole::Caption);
        assert_eq!(StructureType::CodeBlock.to_struct_role(), StructRole::Code);
        assert_eq!(StructureType::BlockQuote.to_struct_role(), StructRole::BlockQuote);
        assert_eq!(StructureType::MathBlock.to_struct_role(), StructRole::Formula);
        assert_eq!(StructureType::TOC.to_struct_role(), StructRole::TOC);
    }

    #[test]
    fn custom_role_names() {
        assert_eq!(StructureType::Chapter.custom_role_name(), Some(b"Chapter".as_slice()));
        assert_eq!(StructureType::Subsection.custom_role_name(), Some(b"Subsection".as_slice()));
        assert_eq!(StructureType::Document.custom_role_name(), None);
        assert_eq!(StructureType::Paragraph.custom_role_name(), None);
    }

    #[test]
    fn alt_text_on_figure() {
        let fig = StructureNode::new(StructureType::Figure, 1, 0).with_alt_text("A diagram");
        assert_eq!(fig.alt_text.as_deref(), Some("A diagram"));
        assert!(fig.is_leaf());
    }

    #[test]
    fn nested_structure() {
        let doc = StructureNode::with_children(
            StructureType::Document,
            vec![
                StructureNode::with_children(
                    StructureType::Section,
                    vec![
                        StructureNode::new(StructureType::Paragraph, 1, 0),
                        StructureNode::new(StructureType::Paragraph, 1, 1),
                    ],
                ),
            ],
        );
        let section = &doc.children[0];
        assert_eq!(section.element_type, StructureType::Section);
        assert_eq!(section.children.len(), 2);
        assert_eq!(section.children[0].mcid, 0);
        assert_eq!(section.children[1].mcid, 1);
    }

    #[test]
    fn empty_structure_tree() {
        let doc = StructureNode::new(StructureType::Document, 0, 0);
        assert!(doc.children.is_empty());
        assert!(doc.is_leaf());
    }
}
