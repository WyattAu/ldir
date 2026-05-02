#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use ldir_ir::sir::v2::module::SIRModuleV2;
use ldir_ir::sir::v2::nodes::*;
use ldir_ir::sir::{BlockType, SIRDocument, SIROpcode, StyleModifier};

pub fn convert_v1_to_v2(doc: &SIRDocument) -> SIRModuleV2 {
    let mut module = SIRModuleV2::new();

    let mut node_map: HashMap<u32, u32> = HashMap::new();
    let mut next_id: u32 = 0;

    let mut style_stack: Vec<NodeType> = Vec::new();
    let mut current_block_v2: Option<u32> = None;

    for instr in doc.iter() {
        let v1_id = instr.entity_id();
        let v2_id = next_id;
        next_id += 1;
        node_map.insert(v1_id, v2_id);

        let parent_v2 = if instr.is_root() {
            None
        } else {
            node_map.get(&instr.parent_id()).copied()
        };

        match instr.opcode() {
            SIROpcode::PushBlock => {
                let payload = doc.payload().get(instr.payload_offset(), 1);
                let block_type = payload.and_then(|bytes| BlockType::from_u8(bytes[0]));

                let node_type = match block_type {
                    Some(BlockType::Document) => NodeType::Document,
                    Some(BlockType::Paragraph) => NodeType::Paragraph,
                    Some(BlockType::Heading) => {
                        let level_payload = doc.payload().get(instr.payload_offset() + 1, 4);
                        let level = level_payload
                            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                            .unwrap_or(1);
                        match level {
                            0 => NodeType::Part,
                            1 => NodeType::Chapter,
                            2 => NodeType::Section,
                            3 => NodeType::Subsection,
                            _ => NodeType::Subsubsection,
                        }
                    }
                    Some(BlockType::List) => NodeType::List {
                        list_type: ListType::Unordered,
                        ordered: false,
                        start: None,
                    },
                    Some(BlockType::Math) => {
                        let numbered = doc
                            .payload()
                            .get(instr.payload_offset() + 1, 1)
                            .map(|b| b[0] == 1)
                            .unwrap_or(false);
                        NodeType::MathBlock {
                            math_type: MathType::Equation,
                            numbered,
                        }
                    }
                    Some(BlockType::Code) => NodeType::CodeBlock { language: None },
                    Some(BlockType::BlockQuote) => NodeType::BlockQuote,
                    Some(BlockType::ThematicBreak) => NodeType::ThematicBreak,
                    Some(BlockType::Image) => NodeType::Image {
                        source: String::new(),
                        alt: String::new(),
                        width: None,
                        height: None,
                    },
                    Some(BlockType::Table) => {
                        let num_cols = doc
                            .payload()
                            .get(instr.payload_offset() + 1, 4)
                            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                            .unwrap_or(0) as usize;
                        NodeType::Table {
                            col_specs: Vec::new(),
                            num_cols,
                        }
                    }
                    Some(BlockType::TableRow) => {
                        let is_header = doc
                            .payload()
                            .get(instr.payload_offset() + 1, 1)
                            .map(|b| b[0] == 1)
                            .unwrap_or(false);
                        NodeType::TableRow { is_header }
                    }
                    Some(BlockType::TableCell) => NodeType::TableCell {
                        colspan: 1,
                        rowspan: 1,
                    },
                    Some(BlockType::Footnote) => {
                        let content = doc
                            .payload_text(instr)
                            .unwrap_or_default()
                            .trim_end_matches('\0')
                            .to_string();
                        NodeType::Footnote { content }
                    }
                    Some(BlockType::FootnoteBlock) => NodeType::FootnoteBlock,
                    Some(BlockType::Figure) => NodeType::Figure {
                        placement: FloatPlacement::Here,
                    },
                    None => NodeType::Paragraph,
                };

                let mut node = Node::new(v2_id, node_type);
                if let Some(pid) = parent_v2 {
                    node = node.with_parent(pid);
                }

                current_block_v2 = Some(v2_id);
                module.body.push(node);
            }

            SIROpcode::SetContent => {
                let text = doc
                    .payload_text(instr)
                    .unwrap_or_default()
                    .trim_end_matches('\0')
                    .to_string();

                if !style_stack.is_empty() {
                    let style_node_type = style_stack.last().unwrap().clone();
                    let mut style_node = Node::new(v2_id, style_node_type);
                    if let Some(pid) = parent_v2.or(current_block_v2) {
                        style_node = style_node.with_parent(pid);
                    }

                    let text_node_id = next_id;
                    next_id += 1;
                    let mut text_node = Node::new(
                        text_node_id,
                        NodeType::Text {
                            content: text.clone(),
                        },
                    );
                    text_node = text_node.with_parent(v2_id);
                    style_node.add_child(text_node_id);

                    module.body.push(style_node);
                    module.body.push(text_node);
                } else {
                    let mut node = Node::new(
                        v2_id,
                        NodeType::Text {
                            content: text.clone(),
                        },
                    );
                    if let Some(pid) = parent_v2.or(current_block_v2) {
                        node = node.with_parent(pid);
                    }
                    module.body.push(node);
                }
            }

            SIROpcode::ApplyStyle => {
                let packed = instr.payload_offset();
                let (modifiers, is_push) = StyleModifier::from_packed(packed);

                if is_push {
                    if modifiers.contains(StyleModifier::BOLD) {
                        style_stack.push(NodeType::Bold);
                    } else if modifiers.contains(StyleModifier::ITALIC) {
                        style_stack.push(NodeType::Italic);
                    } else if modifiers.contains(StyleModifier::MONO) {
                        style_stack.push(NodeType::Mono);
                    } else if modifiers.contains(StyleModifier::UNDERLINE) {
                        style_stack.push(NodeType::Underline);
                    } else if modifiers.contains(StyleModifier::STRIKE) {
                        style_stack.push(NodeType::Strikethrough);
                    } else if modifiers.contains(StyleModifier::SMALL_CAPS) {
                        style_stack.push(NodeType::SmallCaps);
                    } else {
                        style_stack.push(NodeType::Styled {
                            style_name: format!("style_{}", v1_id),
                        });
                    }
                } else {
                    style_stack.pop();
                }
                continue;
            }

            SIROpcode::InsertMath => {
                let text = doc
                    .payload_text(instr)
                    .unwrap_or_default()
                    .trim_end_matches('\0')
                    .to_string();
                let mut node = Node::new(v2_id, NodeType::MathInline { content: text });
                if let Some(pid) = parent_v2.or(current_block_v2) {
                    node = node.with_parent(pid);
                }
                module.body.push(node);
            }

            SIROpcode::LinkData => {
                let url = doc
                    .payload_text(instr)
                    .unwrap_or_default()
                    .trim_end_matches('\0')
                    .to_string();
                let mut node = Node::new(v2_id, NodeType::Link { url, title: None });
                if let Some(pid) = parent_v2.or(current_block_v2) {
                    node = node.with_parent(pid);
                }
                module.body.push(node);
            }
        }
    }

    let parent_ids: Vec<(u32, Option<u32>)> =
        module.body.iter().map(|n| (n.id, n.parent_id)).collect();
    for (child_id, parent_opt) in parent_ids {
        if let Some(pid) = parent_opt
            && let Some(parent) = module.body.get_mut(pid)
        {
            parent.add_child(child_id);
        }
    }

    module
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::{ROOT_SENTINEL, SIRInstruction};

    #[test]
    fn test_convert_empty_doc() {
        let doc = SIRDocument::new();
        let module = convert_v1_to_v2(&doc);
        assert!(module.body.is_empty());
    }

    #[test]
    fn test_convert_simple_document() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 1, 0, 0),
            &[BlockType::Paragraph as u8],
        );
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0),
            b"Hello world",
        );

        let module = convert_v1_to_v2(&doc);
        assert_eq!(module.body.len(), 3);
        let roots = module.body.roots();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], 0);
    }

    #[test]
    fn test_convert_heading() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        let mut heading_payload = vec![BlockType::Heading as u8];
        heading_payload.extend_from_slice(&1u32.to_le_bytes());
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 1, 0, 0),
            &heading_payload,
        );
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0),
            b"Title",
        );

        let module = convert_v1_to_v2(&doc);
        assert!(module.body.get(1).is_some());
        if let Some(node) = module.body.get(1) {
            assert!(matches!(node.node_type, NodeType::Chapter));
        }
    }

    #[test]
    fn test_convert_parent_child() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 1, 0, 0),
            &[BlockType::Paragraph as u8],
        );
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0),
            b"Hello",
        );

        let module = convert_v1_to_v2(&doc);
        let doc_node = module.body.get(0).unwrap();
        assert!(doc_node.child_ids.contains(&1));
        let para_node = module.body.get(1).unwrap();
        assert!(para_node.child_ids.contains(&2));
    }

    #[test]
    fn test_convert_text_content() {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::PushBlock, 1, 0, 0),
            &[BlockType::Paragraph as u8],
        );
        doc.push_with_payload(
            SIRInstruction::new(SIROpcode::SetContent, 2, 1, 0),
            b"Test content",
        );

        let module = convert_v1_to_v2(&doc);
        let text_node = module.body.get(2).unwrap();
        assert_eq!(
            text_node.node_type,
            NodeType::Text {
                content: "Test content".to_string()
            }
        );
    }
}
