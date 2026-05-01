//! S-IR instruction type (13-byte wire format, REQ-3.1.2).
//!
//! Each instruction is a 13-byte fixed-cost header:
//! - OpCode: 1 byte
//! - EntityID: 4 bytes (u32)
//! - ParentID: 4 bytes (u32, sentinel 0xFFFFFFFF for root)
//! - PayloadOffset: 4 bytes (u32)
//!
//! Matches Lean 4 `SIRInstruction` structure in `ProofIRWellformedness.lean`:
//! ```lean
//! structure SIRInstruction where
//!   opcode : SIROpcode
//!   entityId : Nat
//!   parentId : Nat
//!   payloadOffset : Nat
//! ```

use crate::sir::opcode::SIROpcode;

/// Sentinel parent ID indicating the root entity.
///
/// Matches Lean 4 `rootSentinel = 0xFFFFFFFF` and `ldir_core::fp266::ROOT_SENTINEL`.
pub const ROOT_SENTINEL: u32 = 0xFFFF_FFFF;

/// Entity identifier type (32-bit generation index, REQ-3.1.6).
///
/// Maximum capacity: 2^32 nodes per document.
pub type EntityId = u32;

/// Size of a single S-IR instruction in the wire format (REQ-3.1.2).
///
/// OpCode(1) + EntityID(4) + ParentID(4) + PayloadOffset(4) = 13 bytes.
pub const INSTRUCTION_WIRE_SIZE: usize = 13;

/// Atomic S-IR operation with 13-byte wire-format header (REQ-3.1.2).
///
/// Wire layout (C repr, big-endian on wire):
/// ```text
/// Offset  Size  Field
/// 0       1     opcode (u8)
/// 1       4     entity_id (u32)
/// 5       4     parent_id (u32)
/// 9       4     payload_offset (u32)
/// ```
///
/// # Well-Formedness Constraints
///
/// Per DEF-004 (WF-SIR):
/// - **AX-001**: `entity_id` must be unique within the document.
/// - **AX-002**: `parent_id` must reference an existing entity or be `ROOT_SENTINEL`.
/// - **AX-003**: The parent graph must be acyclic.
/// - **AX-004**: `payload_offset` must be within the payload region bounds.
///
/// # Examples
///
/// ```
/// use ldir_ir::sir::{SIRInstruction, SIROpcode, ROOT_SENTINEL};
///
/// let root = SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0);
/// assert_eq!(root.entity_id(), 0);
/// assert_eq!(root.parent_id(), ROOT_SENTINEL);
/// assert!(root.is_root());
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[repr(C)]
#[rkyv(attr(derive(Debug, Clone, Copy, PartialEq, Eq, Hash)))]
pub struct SIRInstruction {
    /// Operation discriminator (1 byte).
    opcode: SIROpcode,
    /// Padding for alignment (3 bytes, wire format unused).
    _pad0: [u8; 3],
    /// Unique entity identifier (4 bytes).
    entity_id: EntityId,
    /// Parent entity reference or `ROOT_SENTINEL` (4 bytes).
    parent_id: EntityId,
    /// Offset into the contiguous payload region (4 bytes).
    payload_offset: u32,
}

impl SIRInstruction {
    /// Create a new S-IR instruction.
    ///
    /// # Arguments
    ///
    /// * `opcode` - The operation to perform.
    /// * `entity_id` - Unique entity identifier (must be unique per document, AX-001).
    /// * `parent_id` - Parent entity reference or `ROOT_SENTINEL` for root.
    /// * `payload_offset` - Offset into the payload region (must be in-bounds, AX-004).
    #[inline]
    pub const fn new(
        opcode: SIROpcode,
        entity_id: EntityId,
        parent_id: EntityId,
        payload_offset: u32,
    ) -> Self {
        Self {
            opcode,
            _pad0: [0; 3],
            entity_id,
            parent_id,
            payload_offset,
        }
    }

    /// Get the operation discriminator.
    #[inline]
    pub const fn opcode(&self) -> SIROpcode {
        self.opcode
    }

    /// Get the unique entity identifier (REQ-3.1.6).
    #[inline]
    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Get the parent entity reference.
    ///
    /// Returns `ROOT_SENTINEL` (0xFFFFFFFF) for the root entity.
    #[inline]
    pub const fn parent_id(&self) -> EntityId {
        self.parent_id
    }

    /// Get the offset into the payload region.
    #[inline]
    pub const fn payload_offset(&self) -> u32 {
        self.payload_offset
    }

    /// Check if this instruction represents the root entity.
    ///
    /// Root entities have `parent_id == ROOT_SENTINEL` (DEF-004 cond. 5).
    #[inline]
    pub const fn is_root(&self) -> bool {
        self.parent_id == ROOT_SENTINEL
    }

    /// Get the size of this instruction in the wire format.
    ///
    /// Always returns 13 bytes per REQ-3.1.2.
    #[inline]
    pub const fn wire_size() -> usize {
        INSTRUCTION_WIRE_SIZE
    }
}

impl Default for SIRInstruction {
    fn default() -> Self {
        Self::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_size() {
        assert_eq!(std::mem::size_of::<SIRInstruction>(), 16);
        assert_eq!(INSTRUCTION_WIRE_SIZE, 13);
    }

    #[test]
    fn test_new_root_instruction() {
        let instr = SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0);
        assert_eq!(instr.opcode(), SIROpcode::PushBlock);
        assert_eq!(instr.entity_id(), 0);
        assert_eq!(instr.parent_id(), ROOT_SENTINEL);
        assert!(instr.is_root());
    }

    #[test]
    fn test_new_child_instruction() {
        let child = SIRInstruction::new(SIROpcode::SetContent, 1, 0, 100);
        assert_eq!(child.entity_id(), 1);
        assert_eq!(child.parent_id(), 0);
        assert!(!child.is_root());
        assert_eq!(child.payload_offset(), 100);
    }

    #[test]
    fn test_const_construction() {
        const ROOT: SIRInstruction =
            SIRInstruction::new(SIROpcode::PushBlock, 42, ROOT_SENTINEL, 0);
        assert_eq!(ROOT.entity_id(), 42);
    }

    #[test]
    fn test_copy_semantics() {
        let a = SIRInstruction::new(SIROpcode::ApplyStyle, 5, 0, 200);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_default() {
        let default = SIRInstruction::default();
        assert_eq!(default.opcode(), SIROpcode::PushBlock);
        assert!(default.is_root());
    }
}
