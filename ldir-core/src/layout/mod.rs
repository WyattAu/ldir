//! Layout pipeline: line-breaking, pagination, and incremental re-layout.
//!
//! - [`linebreak`]: Knuth-Plass line-breaking algorithm (TASK-017)
//! - [`pagination`]: Global page-breaking (TASK-018)
//! - [`incremental`]: Dirty-tracking recompilation (TASK-020)
//!
//! ## References
//! - YP-LAYOUT-KNUTHPLASS-001: Knuth-Plass line breaking theory
//! - YP-LAYOUT-PAGINATION-001: Page breaking theory
//! - THM-KP-OPTIMALITY: DP solution finds globally optimal break set
//! - THM-KP-TERMINATION: Algorithm terminates in O(n²)

pub mod hyphenate;
pub mod incremental;
pub mod linebreak;
pub mod lir_compile;
pub mod multicolumn;
pub mod pagination;
pub mod simd_penalty;
