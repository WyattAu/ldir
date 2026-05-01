use std::collections::HashSet;

use ldir_ir::sir::v2::nodes::{Node, NodeType};
use ldir_ir::sir::v2::SIRModuleV2;

#[derive(Debug, Clone)]
pub enum LinkError {
    NoInputModules,
    LabelCollision(String),
    LinkErrors(Vec<LinkError>),
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::NoInputModules => write!(f, "no input modules to link"),
            LinkError::LabelCollision(label) => write!(f, "label collision: {}", label),
            LinkError::LinkErrors(errors) => {
                for e in errors {
                    writeln!(f, "  {}", e)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for LinkError {}

pub fn link_modules(modules: Vec<SIRModuleV2>) -> Result<SIRModuleV2, LinkError> {
    if modules.is_empty() {
        return Err(LinkError::NoInputModules);
    }
    if modules.len() == 1 {
        return Ok(modules.into_iter().next().unwrap());
    }

    let mut linker = ModuleLinker::new();
    for (i, module) in modules.into_iter().enumerate() {
        linker.add_module(module, i == 0);
    }
    linker.finish()
}

struct ModuleLinker {
    output: SIRModuleV2,
    id_offset: u32,
    errors: Vec<LinkError>,
    used_labels: HashSet<String>,
    used_font_names: HashSet<String>,
    used_color_names: HashSet<String>,
    used_counter_names: HashSet<String>,
    style_names: HashSet<String>,
}

impl ModuleLinker {
    fn new() -> Self {
        Self {
            output: SIRModuleV2::new(),
            id_offset: 0,
            errors: Vec::new(),
            used_labels: HashSet::new(),
            used_font_names: HashSet::new(),
            used_color_names: HashSet::new(),
            used_counter_names: HashSet::new(),
            style_names: HashSet::new(),
        }
    }

    fn add_module(&mut self, module: SIRModuleV2, is_primary: bool) {
        if is_primary {
            self.output.metadata = module.metadata;
            self.output.header = module.header;
        }

        let offset = self.id_offset;
        let max_id = module.body.iter().map(|n| n.id).max().unwrap_or(0);

        for node in module.body.iter() {
            let mut new_node = node.clone();
            new_node.id = node.id + offset;
            new_node.parent_id = node.parent_id.map(|p| p + offset);
            new_node.child_ids = node.child_ids.iter().map(|c| c + offset).collect();

            if let Some(ref label) = new_node.label {
                if self.used_labels.contains(label) {
                    self.errors.push(LinkError::LabelCollision(label.clone()));
                    new_node.label = None;
                } else {
                    self.used_labels.insert(label.clone());
                    let category = if new_node.is_heading() {
                        ldir_ir::sir::v2::annotations::LabelCategory::Section
                    } else if matches!(new_node.node_type, NodeType::MathBlock { .. }) {
                        ldir_ir::sir::v2::annotations::LabelCategory::Equation
                    } else {
                        ldir_ir::sir::v2::annotations::LabelCategory::Custom
                    };
                    self.output.annotations.add_label(label.clone(), new_node.id, category);
                }
            }

            self.output.body.push(new_node);
        }

        for style in module.styles.styles {
            if self.style_names.contains(&style.name) {
                if let Some(existing) = self.output.styles.find_mut(&style.name) {
                    *existing = style;
                }
            } else {
                self.style_names.insert(style.name.clone());
                self.output.styles.styles.push(style);
            }
        }

        for font in module.resources.fonts {
            if !self.used_font_names.contains(&font.name) {
                self.used_font_names.insert(font.name.clone());
                self.output.resources.fonts.push(font);
            }
        }

        for color in module.resources.colors {
            if !self.used_color_names.contains(&color.name) {
                self.used_color_names.insert(color.name.clone());
                self.output.resources.colors.push(color);
            }
        }

        for counter in module.resources.counters {
            if !self.used_counter_names.contains(&counter.name) {
                self.used_counter_names.insert(counter.name.clone());
                self.output.resources.counters.push(counter);
            }
        }

        for r in module.annotations.refs {
            self.output.annotations.add_ref(r.label, r.ref_node_id + offset);
        }

        self.id_offset = offset + max_id + 1;
    }

    fn finish(self) -> Result<SIRModuleV2, LinkError> {
        if !self.errors.is_empty() {
            return Err(LinkError::LinkErrors(self.errors));
        }

        let mut output = self.output;
        let old_roots: Vec<u32> = output.body.roots().to_vec();

        if old_roots.len() <= 1 {
            return Ok(output);
        }

        let doc_id = self.id_offset;
        let mut doc_node = Node::new(doc_id, NodeType::Document);
        for &root_id in &old_roots {
            doc_node.add_child(root_id);
            if let Some(root) = output.body.get_mut(root_id) {
                root.parent_id = Some(doc_id);
            }
        }
        output.body.push(doc_node);
        output.body.rebuild_roots();

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::{Node, NodeType};
    use ldir_ir::sir::v2::styles::{StyleDecl, StyleProperties};
    use ldir_ir::sir::v2::resources::{CounterDecl, CounterFormat, CounterReset, FontDecl, FontWeight, FontStyle, FontSource, ColorDecl, ColorValue};

    fn make_module_with_nodes(nodes: Vec<Node>) -> SIRModuleV2 {
        let mut m = SIRModuleV2::new();
        for node in nodes {
            m.body.push(node);
        }
        m
    }

    // --- Basic linking ---

    #[test]
    fn test_link_empty() {
        let result = link_modules(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_link_single_module() {
        let m = SIRModuleV2::new();
        let result = link_modules(vec![m]).unwrap();
        assert!(result.body.is_empty());
    }

    #[test]
    fn test_link_two_modules() {
        let m1 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Section),
        ]);
        let m2 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Section),
        ]);
        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.body.len(), 3);
    }

    // --- ID remapping ---

    #[test]
    fn test_link_id_remapping() {
        let m1 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Section),
            Node::new(2, NodeType::Paragraph).with_parent(1),
        ]);
        let m2 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Chapter),
        ]);
        let result = link_modules(vec![m1, m2]).unwrap();

        assert!(result.body.get(1).is_some());
        assert!(result.body.get(2).is_some());

        let m2_chapter = result.body.iter().find(|n| matches!(n.node_type, NodeType::Chapter));
        assert!(m2_chapter.is_some());
        assert_ne!(m2_chapter.unwrap().id, 1);
    }

    #[test]
    fn test_link_parent_ids_remapped() {
        let m1 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Section),
            Node::new(2, NodeType::Paragraph).with_parent(1),
        ]);
        let m2 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Section),
            Node::new(2, NodeType::Paragraph).with_parent(1),
        ]);
        let result = link_modules(vec![m1, m2]).unwrap();

        let doc = result.body.get(result.body.roots()[0]).unwrap();
        assert!(matches!(doc.node_type, NodeType::Document));
        assert_eq!(doc.child_ids.len(), 2);
    }

    // --- Style merging ---

    #[test]
    fn test_link_style_merge() {
        let mut m1 = SIRModuleV2::new();
        m1.styles.styles.push(StyleDecl {
            name: "body".to_string(),
            parent: None,
            properties: Default::default(),
        });

        let mut m2 = SIRModuleV2::new();
        m2.styles.styles.push(StyleDecl {
            name: "heading".to_string(),
            parent: Some("body".to_string()),
            properties: Default::default(),
        });

        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.styles.styles.len(), 2);
    }

    #[test]
    fn test_link_style_override() {
        let mut m1 = SIRModuleV2::new();
        m1.styles.styles.push(StyleDecl {
            name: "body".to_string(),
            parent: None,
            properties: StyleProperties {
                font_name: Some("Old".to_string()),
                ..Default::default()
            },
        });

        let mut m2 = SIRModuleV2::new();
        m2.styles.styles.push(StyleDecl {
            name: "body".to_string(),
            parent: None,
            properties: StyleProperties {
                font_name: Some("New".to_string()),
                ..Default::default()
            },
        });

        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.styles.styles.len(), 1);
        assert_eq!(result.styles.styles[0].properties.font_name.as_deref(), Some("New"));
    }

    // --- Resource merging ---

    #[test]
    fn test_link_font_merge() {
        let mut m1 = SIRModuleV2::new();
        m1.resources.fonts.push(FontDecl {
            name: "body".to_string(),
            family: "Inter".to_string(),
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            source: FontSource::System,
            features: Vec::new(),
        });

        let mut m2 = SIRModuleV2::new();
        m2.resources.fonts.push(FontDecl {
            name: "heading".to_string(),
            family: "Inter".to_string(),
            weight: FontWeight::Bold,
            style: FontStyle::Normal,
            source: FontSource::System,
            features: Vec::new(),
        });

        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.resources.fonts.len(), 2);
    }

    #[test]
    fn test_link_font_dedup() {
        let mut m1 = SIRModuleV2::new();
        m1.resources.fonts.push(FontDecl {
            name: "body".to_string(),
            family: "Inter".to_string(),
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            source: FontSource::System,
            features: Vec::new(),
        });

        let mut m2 = SIRModuleV2::new();
        m2.resources.fonts.push(FontDecl {
            name: "body".to_string(),
            family: "Inter".to_string(),
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            source: FontSource::System,
            features: Vec::new(),
        });

        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.resources.fonts.len(), 1);
    }

    #[test]
    fn test_link_color_merge() {
        let mut m1 = SIRModuleV2::new();
        m1.resources.colors.push(ColorDecl {
            name: "primary".to_string(),
            value: ColorValue { r: 0, g: 0, b: 0, a: None },
        });

        let mut m2 = SIRModuleV2::new();
        m2.resources.colors.push(ColorDecl {
            name: "accent".to_string(),
            value: ColorValue { r: 255, g: 0, b: 0, a: None },
        });

        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.resources.colors.len(), 2);
    }

    #[test]
    fn test_link_counter_merge() {
        let mut m1 = SIRModuleV2::new();
        m1.resources.counters.push(CounterDecl {
            name: "section".to_string(),
            format: CounterFormat::Arabic,
            reset_scope: CounterReset::PerChapter,
        });

        let mut m2 = SIRModuleV2::new();
        m2.resources.counters.push(CounterDecl {
            name: "equation".to_string(),
            format: CounterFormat::Arabic,
            reset_scope: CounterReset::PerDocument,
        });

        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.resources.counters.len(), 2);
    }

    // --- Label collision ---

    #[test]
    fn test_link_label_collision() {
        let m1 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Section).with_label("sec:intro"),
        ]);
        let m2 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Section).with_label("sec:intro"),
        ]);

        let result = link_modules(vec![m1, m2]);
        assert!(result.is_err());
        match result.unwrap_err() {
            LinkError::LinkErrors(errs) => assert!(!errs.is_empty()),
            _ => panic!("expected LinkErrors"),
        }
    }

    // --- Root consolidation ---

    #[test]
    fn test_link_root_consolidation() {
        let m1 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Chapter),
        ]);
        let m2 = make_module_with_nodes(vec![
            Node::new(1, NodeType::Chapter),
        ]);

        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.body.roots().len(), 1);
        let root = result.body.get(result.body.roots()[0]).unwrap();
        assert!(matches!(root.node_type, NodeType::Document));
        assert_eq!(root.child_ids.len(), 2);
    }

    // --- Metadata ---

    #[test]
    fn test_link_primary_metadata() {
        let mut m1 = SIRModuleV2::new();
        m1.metadata.title = Some("Primary".to_string());
        let mut m2 = SIRModuleV2::new();
        m2.metadata.title = Some("Secondary".to_string());

        let result = link_modules(vec![m1, m2]).unwrap();
        assert_eq!(result.metadata.title.as_deref(), Some("Primary"));
    }

    // --- Three modules ---

    #[test]
    fn test_link_three_modules() {
        let m1 = make_module_with_nodes(vec![Node::new(1, NodeType::Chapter)]);
        let m2 = make_module_with_nodes(vec![Node::new(1, NodeType::Chapter)]);
        let m3 = make_module_with_nodes(vec![Node::new(1, NodeType::Chapter)]);

        let result = link_modules(vec![m1, m2, m3]).unwrap();
        assert_eq!(result.body.roots().len(), 1);
        let root = result.body.get(result.body.roots()[0]).unwrap();
        assert_eq!(root.child_ids.len(), 3);
    }

    // --- Error display ---

    #[test]
    fn test_link_error_display() {
        let err = LinkError::NoInputModules;
        assert!(!err.to_string().is_empty());
        let err = LinkError::LabelCollision("dup".to_string());
        assert!(err.to_string().contains("dup"));
    }
}
