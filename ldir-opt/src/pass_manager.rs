use crate::{Pass, PassResult};
use ldir_ir::sir::v2::SIRModuleV2;

#[derive(Debug)]
pub struct OptimizationReport {
    pub passes_run: usize,
    pub total_nodes_removed: usize,
    pub total_nodes_added: usize,
    pub iterations: usize,
    pub pass_reports: Vec<PassResult>,
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
    max_iterations: usize,
}

impl PassManager {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            max_iterations: 10,
        }
    }

    pub fn add_pass(&mut self, pass: Box<dyn Pass>) {
        self.passes.push(pass);
    }

    pub fn run(&self, module: &mut SIRModuleV2) -> OptimizationReport {
        let mut report = OptimizationReport {
            passes_run: 0,
            total_nodes_removed: 0,
            total_nodes_added: 0,
            iterations: 1,
            pass_reports: Vec::new(),
        };

        for _ in 0..self.max_iterations {
            let mut changed = false;
            for pass in &self.passes {
                let result = pass.run(module);
                if result.changed {
                    changed = true;
                }
                report.total_nodes_removed += result.nodes_removed;
                report.total_nodes_added += result.nodes_added;
                report.pass_reports.push(result);
                report.passes_run += 1;
            }
            if !changed {
                break;
            }
            report.iterations += 1;
        }

        report
    }

    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeadNodeElimination, DeadStyleElimination};
    use ldir_ir::sir::v2::nodes::{Node, NodeType};

    #[test]
    fn test_pass_manager_new() {
        let pm = PassManager::new();
        assert_eq!(pm.max_iterations, 10);
    }

    #[test]
    fn test_pass_manager_add_and_run() {
        let mut pm = PassManager::new().max_iterations(1);
        pm.add_pass(Box::new(DeadNodeElimination));
        let mut m = SIRModuleV2::new();
        m.body.push(Node::new(1, NodeType::Document));
        let report = pm.run(&mut m);
        assert_eq!(report.passes_run, 1);
    }

    #[test]
    fn test_pass_manager_fixed_point() {
        let mut pm = PassManager::new().max_iterations(5);
        pm.add_pass(Box::new(DeadStyleElimination));
        let mut m = SIRModuleV2::new();
        m.styles.styles.push(ldir_ir::sir::v2::styles::StyleDecl {
            name: "unused".to_string(),
            parent: None,
            properties: Default::default(),
        });
        let report = pm.run(&mut m);
        assert_eq!(report.iterations, 2);
    }

    #[test]
    fn test_optimization_report() {
        let report = OptimizationReport {
            passes_run: 3,
            total_nodes_removed: 10,
            total_nodes_added: 2,
            iterations: 2,
            pass_reports: Vec::new(),
        };
        let debug = format!("{:?}", report);
        assert!(debug.contains("10"));
    }
}
