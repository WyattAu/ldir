use std::collections::{HashMap, HashSet};

use crate::{Pass, PassResult};
use ldir_ir::sir::v2::SIRModuleV2;
use ldir_ir::sir::v2::nodes::NodeType;

// ---------------------------------------------------------------------------
// 1. Dead Node Elimination
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DeadNodeElimination;

impl Pass for DeadNodeElimination {
    fn name(&self) -> &str {
        "dead-node-elimination"
    }

    fn run(&self, module: &mut SIRModuleV2) -> PassResult {
        let mut reachable = HashSet::new();
        let mut queue: Vec<u32> = module.body.roots().to_vec();
        while let Some(id) = queue.pop() {
            if reachable.insert(id)
                && let Some(node) = module.body.get(id)
            {
                for &child_id in &node.child_ids {
                    queue.push(child_id);
                }
            }
        }

        let before = module.body.len();
        module.body.retain(|n| reachable.contains(&n.id));
        let after = module.body.len();
        let removed = before - after;

        module
            .annotations
            .labels
            .retain(|_, info| reachable.contains(&info.node_id));
        module
            .annotations
            .refs
            .retain(|r| reachable.contains(&r.ref_node_id));

        PassResult {
            changed: removed > 0,
            nodes_removed: removed,
            nodes_added: 0,
            details: format!("removed {} unreachable nodes", removed),
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Dead Style Elimination
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DeadStyleElimination;

impl Pass for DeadStyleElimination {
    fn name(&self) -> &str {
        "dead-style-elimination"
    }

    fn run(&self, module: &mut SIRModuleV2) -> PassResult {
        let mut referenced: HashSet<String> =
            module.body.iter().filter_map(|n| n.style.clone()).collect();
        for style in &module.styles.styles {
            if let Some(ref parent) = style.parent {
                referenced.insert(parent.clone());
            }
        }

        let before = module.styles.styles.len();
        module
            .styles
            .styles
            .retain(|s| referenced.contains(&s.name));
        let removed = before - module.styles.styles.len();

        PassResult {
            changed: removed > 0,
            nodes_removed: removed,
            nodes_added: 0,
            details: format!("removed {} unreferenced styles", removed),
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Dead Resource Elimination
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DeadResourceElimination;

impl DeadResourceElimination {
    fn collect_referenced_fonts(module: &SIRModuleV2) -> HashSet<String> {
        let mut names = HashSet::new();
        for style in &module.styles.styles {
            if let Some(ref font) = style.properties.font_name {
                names.insert(font.clone());
            }
        }
        names
    }

    fn collect_referenced_colors(module: &SIRModuleV2) -> HashSet<String> {
        let mut names = HashSet::new();
        for style in &module.styles.styles {
            if let Some(ref c) = style.properties.text_color {
                names.insert(c.clone());
            }
            if let Some(ref c) = style.properties.background_color {
                names.insert(c.clone());
            }
        }
        names
    }

    fn collect_referenced_counters(module: &SIRModuleV2) -> HashSet<String> {
        module
            .body
            .iter()
            .filter_map(|n| n.counter.clone())
            .collect()
    }
}

impl Pass for DeadResourceElimination {
    fn name(&self) -> &str {
        "dead-resource-elimination"
    }

    fn run(&self, module: &mut SIRModuleV2) -> PassResult {
        let fonts = Self::collect_referenced_fonts(module);
        let colors = Self::collect_referenced_colors(module);
        let counters = Self::collect_referenced_counters(module);

        let font_before = module.resources.fonts.len();
        module.resources.fonts.retain(|f| fonts.contains(&f.name));
        let font_removed = font_before - module.resources.fonts.len();

        let color_before = module.resources.colors.len();
        module.resources.colors.retain(|c| colors.contains(&c.name));
        let color_removed = color_before - module.resources.colors.len();

        let counter_before = module.resources.counters.len();
        module
            .resources
            .counters
            .retain(|c| counters.contains(&c.name));
        let counter_removed = counter_before - module.resources.counters.len();

        let total = font_removed + color_removed + counter_removed;

        PassResult {
            changed: total > 0,
            nodes_removed: total,
            nodes_added: 0,
            details: format!(
                "removed {} fonts, {} colors, {} counters",
                font_removed, color_removed, counter_removed
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Empty Block Collapse
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EmptyBlockCollapse;

impl Pass for EmptyBlockCollapse {
    fn name(&self) -> &str {
        "empty-block-collapse"
    }

    fn run(&self, module: &mut SIRModuleV2) -> PassResult {
        let removable: HashSet<u32> = module
            .body
            .iter()
            .filter(|n| {
                matches!(n.node_type, NodeType::Group | NodeType::Document)
                    && n.child_ids.is_empty()
                    && n.label.is_none()
                    && n.style.is_none()
                    && n.counter.is_none()
            })
            .map(|n| n.id)
            .collect();

        if removable.is_empty() {
            return PassResult {
                changed: false,
                nodes_removed: 0,
                nodes_added: 0,
                details: "no empty blocks".into(),
            };
        }

        let before = module.body.len();
        module.body.retain(|n| !removable.contains(&n.id));
        let removed = before - module.body.len();

        PassResult {
            changed: removed > 0,
            nodes_removed: removed,
            nodes_added: 0,
            details: format!("collapsed {} empty blocks", removed),
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Style Inlining
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StyleInlining;

impl StyleInlining {
    fn resolve_style(
        module: &SIRModuleV2,
        name: &str,
    ) -> ldir_ir::sir::v2::styles::StyleProperties {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut current = name.to_string();
        while let Some(decl) = module.styles.find(&current) {
            if !visited.insert(decl.name.clone()) {
                break;
            }
            chain.push(decl.properties.clone());
            match &decl.parent {
                Some(parent) => current = parent.clone(),
                None => break,
            }
        }
        chain.reverse();

        let mut merged = ldir_ir::sir::v2::styles::StyleProperties::default();
        for props in chain {
            Self::merge_properties(&mut merged, &props);
        }
        merged
    }

    fn merge_properties(
        base: &mut ldir_ir::sir::v2::styles::StyleProperties,
        overlay: &ldir_ir::sir::v2::styles::StyleProperties,
    ) {
        if overlay.font_name.is_some() {
            base.font_name = overlay.font_name.clone();
        }
        if overlay.font_size.is_some() {
            base.font_size = overlay.font_size.clone();
        }
        if overlay.font_weight.is_some() {
            base.font_weight = overlay.font_weight;
        }
        if overlay.font_style.is_some() {
            base.font_style = overlay.font_style.clone();
        }
        if overlay.text_color.is_some() {
            base.text_color = overlay.text_color.clone();
        }
        if overlay.background_color.is_some() {
            base.background_color = overlay.background_color.clone();
        }
        if overlay.line_height.is_some() {
            base.line_height = overlay.line_height;
        }
        if overlay.paragraph_indent.is_some() {
            base.paragraph_indent = overlay.paragraph_indent.clone();
        }
        if overlay.space_before.is_some() {
            base.space_before = overlay.space_before.clone();
        }
        if overlay.space_after.is_some() {
            base.space_after = overlay.space_after.clone();
        }
        if overlay.text_align.is_some() {
            base.text_align = overlay.text_align;
        }
        if overlay.keep_with_next.is_some() {
            base.keep_with_next = overlay.keep_with_next;
        }
        if overlay.page_break_before.is_some() {
            base.page_break_before = overlay.page_break_before;
        }
        if overlay.first_line_indent.is_some() {
            base.first_line_indent = overlay.first_line_indent.clone();
        }
        if overlay.margins.is_some() {
            base.margins = overlay.margins.clone();
        }
    }
}

impl Pass for StyleInlining {
    fn name(&self) -> &str {
        "style-inlining"
    }

    fn run(&self, module: &mut SIRModuleV2) -> PassResult {
        let style_refs: Vec<(u32, String)> = module
            .body
            .iter()
            .filter_map(|n| n.style.as_ref().map(|s| (n.id, s.clone())))
            .collect();

        let mut inlined = 0;
        for (node_id, style_name) in style_refs {
            let resolved = Self::resolve_style(module, &style_name);
            let flat_name = format!("__inlined_{}", style_name);
            let flat_style = ldir_ir::sir::v2::styles::StyleDecl {
                name: flat_name.clone(),
                parent: None,
                properties: resolved,
            };

            let already_exists = module
                .styles
                .find(&flat_name)
                .map(|s| s.parent.is_none())
                .unwrap_or(false);

            if !already_exists {
                module.styles.styles.push(flat_style);
            }
            if let Some(n) = module.body.get_mut(node_id) {
                n.style = Some(flat_name);
            }
            inlined += 1;
        }

        PassResult {
            changed: inlined > 0,
            nodes_removed: 0,
            nodes_added: inlined,
            details: format!("inlined {} styles", inlined),
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Counter Propagation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CounterPropagation;

impl Pass for CounterPropagation {
    fn name(&self) -> &str {
        "counter-propagation"
    }

    fn run(&self, module: &mut SIRModuleV2) -> PassResult {
        let counter_decls: HashMap<String, _> = module
            .resources
            .counters
            .iter()
            .map(|c| (c.name.clone(), c.reset_scope))
            .collect();

        let mut counter_values: HashMap<String, u32> = HashMap::new();
        let mut propagated = 0;

        let order: Vec<u32> = module.body.iter().map(|n| n.id).collect();
        for id in order {
            let node = match module.body.get(id) {
                Some(n) => n.clone(),
                None => continue,
            };

            if let Some(ref counter_name) = node.counter {
                let reset_scope = counter_decls
                    .get(counter_name)
                    .copied()
                    .unwrap_or(ldir_ir::sir::v2::resources::CounterReset::PerDocument);

                let should_reset = match reset_scope {
                    ldir_ir::sir::v2::resources::CounterReset::PerDocument => false,
                    ldir_ir::sir::v2::resources::CounterReset::Never => false,
                    _ => false,
                };

                if should_reset {
                    counter_values.insert(counter_name.clone(), 0);
                }

                let value = counter_values.entry(counter_name.clone()).or_insert(0);
                *value += 1;

                if let Some(n) = module.body.get_mut(id) {
                    n.counter = Some(format!("{}:{}", counter_name, value));
                }
                propagated += 1;
            }
        }

        PassResult {
            changed: propagated > 0,
            nodes_removed: 0,
            nodes_added: 0,
            details: format!("propagated {} counters", propagated),
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Label Deduplication
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LabelDeduplication;

impl Pass for LabelDeduplication {
    fn name(&self) -> &str {
        "label-deduplication"
    }

    fn run(&self, module: &mut SIRModuleV2) -> PassResult {
        let mut seen = HashSet::new();
        let mut duplicates = 0;

        for node in module.body.iter_mut() {
            if let Some(ref label) = node.label
                && !seen.insert(label.clone())
            {
                node.label = None;
                duplicates += 1;
            }
        }

        module
            .annotations
            .labels
            .retain(|label, _| seen.contains(label.as_str()));

        PassResult {
            changed: duplicates > 0,
            nodes_removed: 0,
            nodes_added: 0,
            details: format!("removed {} duplicate labels", duplicates),
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Text Node Merging
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TextNodeMerging;

impl Pass for TextNodeMerging {
    fn name(&self) -> &str {
        "text-node-merging"
    }

    fn run(&self, module: &mut SIRModuleV2) -> PassResult {
        let mut to_remove: HashSet<u32> = HashSet::new();
        let mut merges = 0;

        let parent_children: HashMap<u32, Vec<u32>> = module
            .body
            .iter()
            .filter(|n| !n.child_ids.is_empty())
            .map(|n| (n.id, n.child_ids.clone()))
            .collect();

        for children in parent_children.values() {
            let mut i = 0;
            while i + 1 < children.len() {
                let a_id = children[i];
                let b_id = children[i + 1];
                let a = module.body.get(a_id);
                let b = module.body.get(b_id);

                if let (Some(a_node), Some(b_node)) = (a, b)
                    && let (
                        NodeType::Text { content: a_content },
                        NodeType::Text { content: b_content },
                    ) = (&a_node.node_type, &b_node.node_type)
                {
                    let merged_content = format!("{}{}", a_content, b_content);
                    if let Some(n) = module.body.get_mut(a_id) {
                        n.node_type = NodeType::Text {
                            content: merged_content,
                        };
                    }
                    to_remove.insert(b_id);
                    merges += 1;
                    i += 2;
                    continue;
                }
                i += 1;
            }
        }

        if merges > 0 {
            module.body.retain(|n| !to_remove.contains(&n.id));
        }

        PassResult {
            changed: merges > 0,
            nodes_removed: merges,
            nodes_added: 0,
            details: format!("merged {} text node pairs", merges),
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::{Node, NodeType};
    use ldir_ir::sir::v2::resources::{CounterDecl, CounterFormat, CounterReset};
    use ldir_ir::sir::v2::styles::{StyleDecl, StyleProperties};

    // --- Dead Node Elimination ---

    #[test]
    fn test_dead_node_elimination_removes_unreachable() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Document));
        m.body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        m.body.push(
            Node::new(
                99,
                NodeType::Text {
                    content: "orphan".to_string(),
                },
            )
            .with_parent(1),
        );
        if let Some(d) = m.body.get_mut(1) {
            d.add_child(2);
        }

        let pass = DeadNodeElimination;
        let result = pass.run(&mut m);
        assert!(result.changed);
        assert_eq!(result.nodes_removed, 1);
        assert_eq!(m.body.len(), 2);
        assert!(m.body.get(99).is_none());
    }

    #[test]
    fn test_dead_node_elimination_no_change() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Document));
        m.body
            .push(Node::new(2, NodeType::Paragraph).with_parent(1));
        if let Some(d) = m.body.get_mut(1) {
            d.add_child(2);
        }

        let pass = DeadNodeElimination;
        let result = pass.run(&mut m);
        assert!(!result.changed);
        assert_eq!(m.body.len(), 2);
    }

    #[test]
    fn test_dead_node_elimination_deep_tree() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Document));
        m.body.push(Node::new(2, NodeType::Section).with_parent(1));
        m.body
            .push(Node::new(3, NodeType::Paragraph).with_parent(2));
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "deep".to_string(),
                },
            )
            .with_parent(3),
        );
        m.body.push(
            Node::new(
                50,
                NodeType::Text {
                    content: "orphan".to_string(),
                },
            )
            .with_parent(1),
        );
        for (parent, child) in [(1, 2), (2, 3), (3, 4)] {
            if let Some(p) = m.body.get_mut(parent) {
                p.add_child(child);
            }
        }

        DeadNodeElimination.run(&mut m);
        assert_eq!(m.body.len(), 4);
        assert!(m.body.get(50).is_none());
    }

    // --- Dead Style Elimination ---

    #[test]
    fn test_dead_style_elimination_removes_unused() {
        let mut m = SIRModuleV2::new();
        m.styles.styles.push(StyleDecl {
            name: "used".to_string(),
            parent: None,
            properties: Default::default(),
        });
        m.styles.styles.push(StyleDecl {
            name: "unused".to_string(),
            parent: None,
            properties: Default::default(),
        });
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_style("used"));

        let pass = DeadStyleElimination;
        let result = pass.run(&mut m);
        assert!(result.changed);
        assert_eq!(m.styles.styles.len(), 1);
        assert_eq!(m.styles.styles[0].name, "used");
    }

    #[test]
    fn test_dead_style_elimination_keeps_parent_chain() {
        let mut m = SIRModuleV2::new();
        m.styles.styles.push(StyleDecl {
            name: "body".to_string(),
            parent: None,
            properties: Default::default(),
        });
        m.styles.styles.push(StyleDecl {
            name: "heading".to_string(),
            parent: Some("body".to_string()),
            properties: Default::default(),
        });
        m.body
            .push(Node::new(1, NodeType::Section).with_style("heading"));

        DeadStyleElimination.run(&mut m);
        assert_eq!(m.styles.styles.len(), 2);
    }

    // --- Dead Resource Elimination ---

    #[test]
    fn test_dead_resource_elimination() {
        let mut m = SIRModuleV2::new();
        m.resources
            .fonts
            .push(ldir_ir::sir::v2::resources::FontDecl {
                name: "used".to_string(),
                family: "Inter".to_string(),
                weight: ldir_ir::sir::v2::resources::FontWeight::Regular,
                style: ldir_ir::sir::v2::resources::FontStyle::Normal,
                source: ldir_ir::sir::v2::resources::FontSource::System,
                features: Vec::new(),
            });
        m.resources
            .fonts
            .push(ldir_ir::sir::v2::resources::FontDecl {
                name: "unused".to_string(),
                family: "Unused".to_string(),
                weight: ldir_ir::sir::v2::resources::FontWeight::Regular,
                style: ldir_ir::sir::v2::resources::FontStyle::Normal,
                source: ldir_ir::sir::v2::resources::FontSource::System,
                features: Vec::new(),
            });
        m.styles.styles.push(StyleDecl {
            name: "body".to_string(),
            parent: None,
            properties: StyleProperties {
                font_name: Some("used".to_string()),
                ..Default::default()
            },
        });

        let pass = DeadResourceElimination;
        let result = pass.run(&mut m);
        assert!(result.changed);
        assert_eq!(m.resources.fonts.len(), 1);
    }

    #[test]
    fn test_dead_resource_elimination_counters() {
        let mut m = SIRModuleV2::new();
        m.resources.counters.push(CounterDecl {
            name: "section".to_string(),
            format: CounterFormat::Arabic,
            reset_scope: CounterReset::PerChapter,
        });
        m.resources.counters.push(CounterDecl {
            name: "unused_counter".to_string(),
            format: CounterFormat::Arabic,
            reset_scope: CounterReset::PerDocument,
        });
        m.body
            .push(Node::new(1, NodeType::Section).with_counter("section"));

        DeadResourceElimination.run(&mut m);
        assert_eq!(m.resources.counters.len(), 1);
        assert_eq!(m.resources.counters[0].name, "section");
    }

    // --- Empty Block Collapse ---

    #[test]
    fn test_empty_block_collapse() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Document));
        m.body.push(Node::new(2, NodeType::Group));
        m.body
            .push(Node::new(3, NodeType::Group).with_label("keep-me"));
        m.body.push(Node::new(4, NodeType::Section).with_parent(1));
        if let Some(d) = m.body.get_mut(1) {
            d.add_child(4);
        }

        let pass = EmptyBlockCollapse;
        let result = pass.run(&mut m);
        assert!(result.changed);
        assert_eq!(result.nodes_removed, 1);
        assert!(m.body.get(3).is_some());
        assert!(m.body.get(2).is_none());
    }

    #[test]
    fn test_empty_block_collapse_keeps_document_with_children() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Document));
        m.body.push(Node::new(2, NodeType::Section).with_parent(1));
        if let Some(d) = m.body.get_mut(1) {
            d.add_child(2);
        }

        let result = EmptyBlockCollapse.run(&mut m);
        assert!(!result.changed);
        assert_eq!(m.body.len(), 2);
    }

    // --- Style Inlining ---

    #[test]
    fn test_style_inlining_flat() {
        let mut m = SIRModuleV2::new();
        m.styles.styles.push(StyleDecl {
            name: "body".to_string(),
            parent: None,
            properties: StyleProperties {
                font_name: Some("Inter".to_string()),
                ..Default::default()
            },
        });
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_style("body"));

        let pass = StyleInlining;
        let result = pass.run(&mut m);
        assert!(result.changed);
        assert_eq!(m.styles.styles.len(), 2);
        assert_eq!(m.styles.styles[1].parent, None);
        assert_eq!(
            m.styles.styles[1].properties.font_name.as_deref(),
            Some("Inter")
        );
    }

    #[test]
    fn test_style_inlining_inheritance() {
        let mut m = SIRModuleV2::new();
        m.styles.styles.push(StyleDecl {
            name: "base".to_string(),
            parent: None,
            properties: StyleProperties {
                font_name: Some("Base".to_string()),
                font_size: Some(ldir_ir::sir::v2::metadata::Dimension::Pt(12.0)),
                ..Default::default()
            },
        });
        m.styles.styles.push(StyleDecl {
            name: "derived".to_string(),
            parent: Some("base".to_string()),
            properties: StyleProperties {
                font_name: Some("Derived".to_string()),
                ..Default::default()
            },
        });
        m.body
            .push(Node::new(1, NodeType::Paragraph).with_style("derived"));

        StyleInlining.run(&mut m);
        let inlined = m.styles.find("__inlined_derived").unwrap();
        assert!(inlined.parent.is_none());
        assert_eq!(inlined.properties.font_name.as_deref(), Some("Derived"));
        assert_eq!(
            inlined.properties.font_size,
            Some(ldir_ir::sir::v2::metadata::Dimension::Pt(12.0))
        );
    }

    // --- Counter Propagation ---

    #[test]
    fn test_counter_propagation() {
        let mut m = SIRModuleV2::new();
        m.resources.counters.push(CounterDecl {
            name: "section".to_string(),
            format: CounterFormat::Arabic,
            reset_scope: CounterReset::PerDocument,
        });
        m.body
            .push(Node::new(1, NodeType::Section).with_counter("section"));
        m.body
            .push(Node::new(2, NodeType::Section).with_counter("section"));

        let pass = CounterPropagation;
        let result = pass.run(&mut m);
        assert!(result.changed);
        assert_eq!(m.body.get(1).unwrap().counter.as_deref(), Some("section:1"));
        assert_eq!(m.body.get(2).unwrap().counter.as_deref(), Some("section:2"));
    }

    // --- Label Deduplication ---

    #[test]
    fn test_label_deduplication() {
        let mut m = SIRModuleV2::new();
        m.body
            .push(Node::new(1, NodeType::Section).with_label("dup"));
        m.body
            .push(Node::new(2, NodeType::Section).with_label("dup"));
        m.annotations.add_label(
            "dup".to_string(),
            1,
            ldir_ir::sir::v2::annotations::LabelCategory::Section,
        );
        m.annotations.add_label(
            "dup".to_string(),
            2,
            ldir_ir::sir::v2::annotations::LabelCategory::Section,
        );

        let pass = LabelDeduplication;
        let result = pass.run(&mut m);
        assert!(result.changed);
        assert_eq!(m.body.get(1).unwrap().label.as_deref(), Some("dup"));
        assert!(m.body.get(2).unwrap().label.is_none());
    }

    #[test]
    fn test_label_deduplication_no_duplicates() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Section).with_label("a"));
        m.body.push(Node::new(2, NodeType::Section).with_label("b"));

        let result = LabelDeduplication.run(&mut m);
        assert!(!result.changed);
    }

    // --- Text Node Merging ---

    #[test]
    fn test_text_node_merging() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Paragraph));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "Hello ".to_string(),
                },
            )
            .with_parent(1),
        );
        m.body.push(
            Node::new(
                3,
                NodeType::Text {
                    content: "world".to_string(),
                },
            )
            .with_parent(1),
        );
        if let Some(p) = m.body.get_mut(1) {
            p.add_child(2);
            p.add_child(3);
        }

        let pass = TextNodeMerging;
        let result = pass.run(&mut m);
        assert!(result.changed);
        assert_eq!(result.nodes_removed, 1);
        assert_eq!(m.body.len(), 2);
        if let NodeType::Text { content } = &m.body.get(2).unwrap().node_type {
            assert_eq!(content, "Hello world");
        } else {
            panic!("expected Text node");
        }
    }

    #[test]
    fn test_text_node_merging_non_adjacent() {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Paragraph));
        m.body.push(
            Node::new(
                2,
                NodeType::Text {
                    content: "A".to_string(),
                },
            )
            .with_parent(1),
        );
        m.body.push(Node::new(3, NodeType::Bold).with_parent(1));
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "B".to_string(),
                },
            )
            .with_parent(1),
        );
        if let Some(p) = m.body.get_mut(1) {
            p.add_child(2);
            p.add_child(3);
            p.add_child(4);
        }

        let result = TextNodeMerging.run(&mut m);
        assert!(!result.changed);
        assert_eq!(m.body.len(), 4);
    }

    // --- Integration: full pipeline ---

    #[test]
    fn test_full_optimization_pipeline() {
        let mut m = SIRModuleV2::new();
        m.styles.styles.push(StyleDecl {
            name: "body".to_string(),
            parent: None,
            properties: Default::default(),
        });
        m.styles.styles.push(StyleDecl {
            name: "orphan_style".to_string(),
            parent: None,
            properties: Default::default(),
        });
        m.body.push(Node::new(1, NodeType::Document));
        m.body.push(
            Node::new(2, NodeType::Section)
                .with_parent(1)
                .with_label("sec:a"),
        );
        m.body.push(
            Node::new(3, NodeType::Paragraph)
                .with_parent(2)
                .with_style("body"),
        );
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Hello ".to_string(),
                },
            )
            .with_parent(3),
        );
        m.body.push(
            Node::new(
                5,
                NodeType::Text {
                    content: "world".to_string(),
                },
            )
            .with_parent(3),
        );
        m.body.push(Node::new(6, NodeType::Group).with_parent(1));
        m.body.push(
            Node::new(
                99,
                NodeType::Text {
                    content: "orphan".to_string(),
                },
            )
            .with_parent(1),
        );
        for (parent, child) in [(1, 2), (2, 3), (3, 4), (3, 5)] {
            if let Some(p) = m.body.get_mut(parent) {
                p.add_child(child);
            }
        }

        let pm = crate::default_pass_manager();
        let report = pm.run(&mut m);

        assert!(report.passes_run > 0);
        assert!(report.total_nodes_removed > 0);
        assert!(m.body.get(99).is_none());
        assert!(m.body.get(6).is_none());
    }
}
