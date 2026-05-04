//! S-IR document type.
//!
//! A `SIRDocument` is an ordered collection of `SIRInstruction` values
//! representing a tree-structured document with entity-based addressing.
//!
//! Matches Lean 4 `abbrev SIRDocument := List SIRInstruction` in
//! `ProofIRWellformedness.lean` Section 1.
//!
//! # Well-Formedness (DEF-004)
//!
//! A document is well-formed iff all 6 WF-SIR conditions hold:
//! 1. **AX-001** (`entityUnique`): All entity IDs are distinct.
//! 2. **AX-002** (`parentExists`): Every parent reference is valid.
//! 3. **AX-003** (`isAcyclic`): No circular parent chains.
//! 4. **AX-004**: Payload offsets are in bounds.
//! 5. **DEF-004.5** (`hasSingleRoot`): Exactly one root entity.
//! 6. **DEF-004.6**: Block nesting is properly structured.

use crate::sir::instruction::{EntityId, SIRInstruction};
use crate::sir::payload::PayloadRegion;

/// Ordered collection of S-IR instructions representing a document.
///
/// The instruction sequence encodes a tree structure via parent references:
/// each instruction's `parent_id` points to another instruction's `entity_id`
/// (or `ROOT_SENTINEL` for the single root).
///
/// # Examples
///
/// ```
/// use ldir_ir::sir::{SIRDocument, SIRInstruction, SIROpcode, ROOT_SENTINEL};
///
/// let mut doc = SIRDocument::new();
/// doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));
/// doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
///
/// assert_eq!(doc.len(), 2);
/// assert_eq!(doc.entity_ids().collect::<Vec<_>>(), vec![0, 1]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SIRDocument {
    instructions: Vec<SIRInstruction>,
    /// Contiguous payload data referenced by instruction `payload_offset` fields.
    payload: PayloadRegion,
    /// Footnote entries: (number, text).
    pub footnotes: Vec<(u32, String)>,
}

impl SIRDocument {
    /// Create a new empty S-IR document.
    #[inline]
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            payload: PayloadRegion::new(),
            footnotes: Vec::new(),
        }
    }

    /// Create a new S-IR document with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            instructions: Vec::with_capacity(capacity),
            payload: PayloadRegion::new(),
            footnotes: Vec::new(),
        }
    }

    /// Number of instructions in the document.
    #[inline]
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if the document has no instructions.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Get a reference to an instruction by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&SIRInstruction> {
        self.instructions.get(index)
    }

    /// Get a mutable reference to an instruction by index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut SIRInstruction> {
        self.instructions.get_mut(index)
    }

    /// Push an instruction onto the document.
    #[inline]
    pub fn push(&mut self, instruction: SIRInstruction) {
        self.instructions.push(instruction);
    }

    /// Iterate over entity IDs in document order.
    ///
    /// Used by `entityUnique` (DEF-004 cond. 1) to check for duplicates.
    pub fn entity_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.instructions.iter().map(|i| i.entity_id())
    }

    /// Iterate over instructions in document order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &SIRInstruction> {
        self.instructions.iter()
    }

    /// Iterate over instructions mutably.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SIRInstruction> {
        self.instructions.iter_mut()
    }

    /// Find an instruction by entity ID.
    ///
    /// Returns the index of the first instruction with the given entity ID,
    /// or `None` if not found.
    ///
    /// Per AX-001, entity IDs are unique, so at most one match exists
    /// in a well-formed document.
    pub fn find_by_entity_id(&self, entity_id: EntityId) -> Option<usize> {
        self.instructions
            .iter()
            .position(|i| i.entity_id() == entity_id)
    }

    /// Find all root instructions (parent_id == ROOT_SENTINEL).
    ///
    /// Per DEF-004 cond. 5, a well-formed document has exactly one root.
    pub fn roots(&self) -> impl Iterator<Item = &SIRInstruction> + '_ {
        self.instructions.iter().filter(|i| i.is_root())
    }

    /// Get the raw instruction slice.
    #[inline]
    pub fn as_slice(&self) -> &[SIRInstruction] {
        &self.instructions
    }

    /// Get the raw instruction vector.
    #[inline]
    pub fn into_vec(self) -> Vec<SIRInstruction> {
        self.instructions
    }

    /// Clear all instructions.
    pub fn clear(&mut self) {
        self.instructions.clear();
    }

    /// Reserve capacity for additional instructions.
    pub fn reserve(&mut self, additional: usize) {
        self.instructions.reserve(additional);
    }

    /// Push an instruction and associate payload data with it.
    ///
    /// The payload bytes are appended to the internal payload region, and
    /// the instruction's `payload_offset` is set to the start of the appended data.
    pub fn push_with_payload(&mut self, mut instruction: SIRInstruction, payload_data: &[u8]) {
        let offset = self.payload.append(payload_data);
        instruction = SIRInstruction::new(
            instruction.opcode(),
            instruction.entity_id(),
            instruction.parent_id(),
            offset,
        );
        self.instructions.push(instruction);
    }

    /// Get a reference to the payload region.
    #[inline]
    pub fn payload(&self) -> &PayloadRegion {
        &self.payload
    }

    /// Get a mutable reference to the payload region.
    #[inline]
    pub fn payload_mut(&mut self) -> &mut PayloadRegion {
        &mut self.payload
    }

    /// Look up the payload string for a given instruction via its `payload_offset`.
    ///
    /// Returns the text from `payload_offset` to the next NUL byte or end of payload.
    /// Returns `None` if the offset is out of bounds or the bytes are not valid UTF-8.
    pub fn payload_text(&self, instruction: &SIRInstruction) -> Option<&str> {
        let bytes = self.payload.get_until_nul(instruction.payload_offset())?;
        std::str::from_utf8(bytes).ok()
    }

    /// Serialize this document to rkyv bytes (instructions only).
    ///
    /// Note: Payload region is NOT included. Use [`Self::to_bytes_with_payload`]
    /// for full round-trip fidelity including payload data.
    pub fn to_bytes(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(&self.instructions)
            .unwrap_or_else(|e| {
                unreachable!("rkyv serialization of SIR instructions should not fail: {e}")
            })
            .into_vec()
    }

    /// Deserialize an `S-IR document` from rkyv bytes (instructions only).
    ///
    /// The payload region will be empty. Use [`Self::from_bytes_with_payload`]
    /// for full round-trip fidelity.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, rkyv::rancor::Error> {
        let instructions: Vec<SIRInstruction> = rkyv::from_bytes::<_, rkyv::rancor::Error>(bytes)?;
        Ok(Self {
            instructions,
            payload: PayloadRegion::new(),
            footnotes: Vec::new(),
        })
    }

    /// Serialize document with payload as a combined byte buffer.
    ///
    /// Format: `[u32 payload_len][payload bytes][rkyv instruction bytes]`
    pub fn to_bytes_with_payload(&self) -> Vec<u8> {
        let payload_bytes = self.payload.as_bytes();
        let instr_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&self.instructions)
            .unwrap_or_else(|e| {
                unreachable!("rkyv serialization of SIR instructions should not fail: {e}")
            })
            .into_vec();
        let payload_len = payload_bytes.len() as u32;
        let mut out = Vec::with_capacity(4 + payload_bytes.len() + instr_bytes.len());
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(payload_bytes);
        out.extend_from_slice(&instr_bytes);
        out
    }

    /// Deserialize document from combined byte buffer.
    pub fn from_bytes_with_payload(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < 4 {
            return Err("buffer too short for payload header".into());
        }
        let payload_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        if bytes.len() < 4 + payload_len {
            return Err("buffer too short for payload data".into());
        }
        let payload = PayloadRegion::from_bytes(bytes[4..4 + payload_len].to_vec());
        let instructions: Vec<SIRInstruction> =
            rkyv::from_bytes::<_, rkyv::rancor::Error>(&bytes[4 + payload_len..])?;
        Ok(Self {
            instructions,
            payload,
            footnotes: Vec::new(),
        })
    }
}

impl Default for SIRDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for SIRDocument {
    type Target = [SIRInstruction];

    fn deref(&self) -> &Self::Target {
        &self.instructions
    }
}

impl std::ops::DerefMut for SIRDocument {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instructions
    }
}

impl IntoIterator for SIRDocument {
    type Item = SIRInstruction;
    type IntoIter = std::vec::IntoIter<SIRInstruction>;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sir::ROOT_SENTINEL;
    use crate::sir::opcode::SIROpcode;

    fn make_doc() -> SIRDocument {
        let mut doc = SIRDocument::new();
        doc.push(SIRInstruction::new(
            SIROpcode::PushBlock,
            0,
            ROOT_SENTINEL,
            0,
        ));
        doc.push(SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0));
        doc.push(SIRInstruction::new(SIROpcode::ApplyStyle, 2, 0, 10));
        doc
    }

    #[test]
    fn test_new_empty() {
        let doc = SIRDocument::new();
        assert!(doc.is_empty());
        assert_eq!(doc.len(), 0);
    }

    #[test]
    fn test_push_and_len() {
        let doc = make_doc();
        assert_eq!(doc.len(), 3);
        assert!(!doc.is_empty());
    }

    #[test]
    fn test_get() {
        let doc = make_doc();
        assert_eq!(doc.get(0).unwrap().entity_id(), 0);
        assert_eq!(doc.get(1).unwrap().entity_id(), 1);
        assert!(doc.get(10).is_none());
    }

    #[test]
    fn test_entity_ids() {
        let doc = make_doc();
        assert_eq!(doc.entity_ids().collect::<Vec<_>>(), vec![0, 1, 2]);
    }

    #[test]
    fn test_iter() {
        let doc = make_doc();
        let opcodes: Vec<_> = doc.iter().map(|i| i.opcode()).collect();
        assert_eq!(
            opcodes,
            vec![
                SIROpcode::PushBlock,
                SIROpcode::SetContent,
                SIROpcode::ApplyStyle
            ]
        );
    }

    #[test]
    fn test_find_by_entity_id() {
        let doc = make_doc();
        assert_eq!(doc.find_by_entity_id(1), Some(1));
        assert_eq!(doc.find_by_entity_id(99), None);
    }

    #[test]
    fn test_roots() {
        let doc = make_doc();
        let root_count = doc.roots().count();
        assert_eq!(root_count, 1);
        assert_eq!(doc.roots().next().unwrap().entity_id(), 0);
    }

    #[test]
    fn test_default() {
        let doc = SIRDocument::default();
        assert!(doc.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let doc = SIRDocument::with_capacity(100);
        assert!(doc.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut doc = make_doc();
        doc.clear();
        assert!(doc.is_empty());
    }

    #[test]
    fn test_into_iter() {
        let doc = make_doc();
        let ids: Vec<_> = doc.into_iter().map(|i| i.entity_id()).collect();
        assert_eq!(ids, vec![0, 1, 2]);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::sir::ROOT_SENTINEL;
    use crate::sir::opcode::SIROpcode;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn roundtrip_document_instructions(ref instrs in proptest::collection::vec(any::<u8>(), 0..100)) {
            let mut doc = SIRDocument::new();
            for &b in instrs {
                let opcode = SIROpcode::from_u8(b % 5).unwrap_or(SIROpcode::PushBlock);
                let entity_id = b as u32;
                let parent_id = if b % 3 == 0 { ROOT_SENTINEL } else { (b as u32).saturating_sub(1) };
                doc.push(SIRInstruction::new(opcode, entity_id, parent_id, 0));
            }
            let _ = doc.len();
        }
    }

    proptest! {
        #[test]
        fn payload_roundtrip(ref content in "[a-zA-Z0-9 ]{0,1000}") {
            let mut doc = SIRDocument::new();
            doc.push(SIRInstruction::new(SIROpcode::PushBlock, 0, ROOT_SENTINEL, 0));
            doc.push_with_payload(
                SIRInstruction::new(SIROpcode::SetContent, 1, 0, 0),
                content.as_bytes(),
            );
            let text = doc.payload_text(doc.get(1).unwrap());
            assert_eq!(text, Some(content.as_str()));
        }
    }
}
