//! Entity Component System core module for LDIR.
//!
//! This module implements a sparse-set ECS architecture per
//! [YP-MEMORY-ECS-001](crate#references), providing:
//!
//! - [`entity`] — Entity identifiers and bump allocator with generation tracking
//! - [`component`] — Sparse set and typed component storage (SoA layout)
//! - [`storage`] — World container with type-erased component management
//! - [`arena`] — Simple bump allocator for contiguous value storage
//!
//! # Design Principles
//!
//! - **O(1) component access** via double indirection (THM-ECS-ACCESS)
//! - **Deterministic iteration order** sorted by entity ID (THM-ECS-DETERMINISM)
//! - **Cache-friendly SoA layout** for component data (THM-ECS-CACHE-FRIENDLY)
//! - **No unsafe code** in Phase A (correctness-first)
//! - **Generation-based stale reference detection** (DEF-ENTITY)
//!
//! # References
//!
//! - YP-MEMORY-ECS-001: Entity Component System Memory Architecture
//! - REQ-2.1: ECS architecture with SoA storage
//! - REQ-3.1.6: 32-bit generation indices
//! - REQ-4.1.1: Zero dynamic heap allocations during hot layout pass
//! - REQ-4.1.2: Structure of Arrays layout
//! - REQ-4.1.3: 64-byte cache-line alignment
//! - REQ-4.1.4: No Box/Rc/Arc for document nodes

pub mod arena;
pub mod component;
pub mod entity;
pub mod storage;

pub use arena::Arena;
pub use component::{ComponentId, ComponentStore, SparseSet};
pub use entity::{Entity, EntityAllocator, EntityId};
pub use storage::World;
