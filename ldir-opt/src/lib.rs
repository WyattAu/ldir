#![warn(clippy::unwrap_used, clippy::expect_used)]
#![deny(unsafe_code)]

mod pass_manager;
mod passes;

pub use pass_manager::{OptimizationReport, PassManager};
pub use passes::*;

use ldir_ir::sir::v2::SIRModuleV2;

#[derive(Debug)]
pub struct PassResult {
    pub changed: bool,
    pub nodes_removed: usize,
    pub nodes_added: usize,
    pub details: String,
}

pub trait Pass {
    fn name(&self) -> &str;
    fn run(&self, module: &mut SIRModuleV2) -> PassResult;
}

pub fn all_passes() -> Vec<Box<dyn Pass>> {
    vec![
        Box::new(DeadNodeElimination),
        Box::new(DeadStyleElimination),
        Box::new(DeadResourceElimination),
        Box::new(EmptyBlockCollapse),
        Box::new(StyleInlining),
        Box::new(CounterPropagation),
        Box::new(LabelDeduplication),
        Box::new(TextNodeMerging),
    ]
}

pub fn default_pass_manager() -> PassManager {
    let mut pm = PassManager::new();
    for pass in all_passes() {
        pm.add_pass(pass);
    }
    pm
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldir_ir::sir::v2::nodes::{Node, NodeType};

    fn test_module() -> SIRModuleV2 {
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Document));
        m.body.push(Node::new(2, NodeType::Section).with_parent(1));
        m.body
            .push(Node::new(3, NodeType::Paragraph).with_parent(2));
        m.body.push(
            Node::new(
                4,
                NodeType::Text {
                    content: "Hello".to_string(),
                },
            )
            .with_parent(3),
        );
        if let Some(d) = m.body.get_mut(1) {
            d.add_child(2);
        }
        if let Some(s) = m.body.get_mut(2) {
            s.add_child(3);
        }
        if let Some(p) = m.body.get_mut(3) {
            p.add_child(4);
        }
        m
    }

    #[test]
    fn test_all_passes_returns_eight() {
        assert_eq!(all_passes().len(), 8);
    }

    #[test]
    fn test_default_pass_manager() {
        let pm = default_pass_manager();
        let mut m = test_module();
        let report = pm.run(&mut m);
        assert!(report.passes_run == 8);
    }

    #[test]
    fn test_pass_result_debug() {
        let r = PassResult {
            changed: true,
            nodes_removed: 5,
            nodes_added: 0,
            details: "removed 5".to_string(),
        };
        let debug = format!("{:?}", r);
        assert!(debug.contains("5"));
    }

    #[test]
    fn test_pass_names_are_unique() {
        let passes = all_passes();
        let names: Vec<&str> = passes.iter().map(|p| p.name()).collect();
        let mut seen = std::collections::HashSet::new();
        for name in names {
            assert!(seen.insert(name), "duplicate pass name: {}", name);
        }
    }
}
