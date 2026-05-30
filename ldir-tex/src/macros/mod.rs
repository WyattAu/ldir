pub mod amsmath;
pub mod graphicx;

use std::collections::HashMap;

pub struct MacroRegistry {
    math_symbols: HashMap<&'static str, &'static str>,
    pub graphicx_paths: Vec<String>,
}

impl MacroRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            math_symbols: HashMap::new(),
            graphicx_paths: Vec::new(),
        };
        amsmath::register(&mut reg);
        graphicx::register(&mut reg);
        reg
    }

    pub fn lookup_symbol(&self, cmd: &str) -> Option<&'static str> {
        self.math_symbols.get(cmd).copied()
    }
}
