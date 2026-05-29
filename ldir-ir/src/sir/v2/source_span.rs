use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    pub line: u32,
    pub col: u32,
    pub byte_offset: u32,
    pub len: u32,
}

impl SourceSpan {
    pub fn new(line: u32, col: u32, byte_offset: u32, len: u32) -> Self {
        Self {
            line,
            col,
            byte_offset,
            len,
        }
    }

    pub fn unknown() -> Self {
        Self {
            line: 0,
            col: 0,
            byte_offset: 0,
            len: 0,
        }
    }
}

impl std::fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}
