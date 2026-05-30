//! CRDT-based collaborative editing support.
//!
//! Implements a sequence CRDT for concurrent text editing,
//! enabling multiple users to edit the same document simultaneously
//! without conflicts.

/// Operational Transform fallback for simpler concurrency scenarios.
pub mod ot;

/// Unique identifier for a character in the sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CharId {
    /// Site ID (unique per editor session).
    pub site: u32,
    /// Monotonically increasing counter within a site.
    pub counter: u64,
}

/// A character in the CRDT sequence with positional metadata.
#[derive(Debug, Clone)]
pub struct CrdtChar {
    /// Unique character identifier.
    pub id: CharId,
    /// Unique ID of the character to the left in the ordering.
    pub origin_left: CharId,
    /// Unique ID of the character to the right in the ordering.
    pub origin_right: CharId,
    /// The actual character content.
    pub value: char,
    /// Whether this character has been deleted.
    pub deleted: bool,
}

/// Operations that can be applied to the document.
#[derive(Debug, Clone)]
pub enum CrdtOp {
    /// Insert a character at a position.
    Insert(CrdtChar),
    /// Delete a character by ID.
    Delete(CharId),
}

/// A text operation that can be sent over the network.
#[derive(Debug, Clone)]
pub struct TextOperation {
    /// Unique operation ID.
    pub id: u64,
    /// Site that created this operation.
    pub site: u32,
    /// The operation to apply.
    pub op: CrdtOp,
}

/// CRDT document that supports concurrent editing.
#[derive(Debug, Clone)]
pub struct CrdtDocument {
    /// The sequence of characters (including tombstones).
    chars: Vec<CrdtChar>,
    /// Next counter for this site.
    next_counter: u64,
    /// Site ID for this editor.
    site: u32,
}

impl CrdtDocument {
    /// Create a new empty CRDT document.
    pub fn new(site: u32) -> Self {
        Self {
            chars: Vec::new(),
            next_counter: 0,
            site,
        }
    }

    /// Allocate a new unique CharId.
    fn alloc_id(&mut self) -> CharId {
        let id = CharId {
            site: self.site,
            counter: self.next_counter,
        };
        self.next_counter += 1;
        id
    }

    /// Insert text at a visible index.
    pub fn insert(&mut self, index: usize, text: &str) -> Vec<TextOperation> {
        let mut ops = Vec::new();
        let (mut left_id, right_id) = self.get_origins_at(index);

        for ch in text.chars() {
            let id = self.alloc_id();
            let crdt_char = CrdtChar {
                id,
                origin_left: left_id,
                origin_right: right_id,
                value: ch,
                deleted: false,
            };
            ops.push(TextOperation {
                id: self.next_counter,
                site: self.site,
                op: CrdtOp::Insert(crdt_char.clone()),
            });
            left_id = id;
            self.apply_op(&CrdtOp::Insert(crdt_char));
        }

        ops
    }

    /// Delete `count` characters starting at `index`.
    pub fn delete(&mut self, index: usize, count: usize) -> Vec<TextOperation> {
        let mut ops = Vec::new();
        let visible: Vec<usize> = self
            .chars
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.deleted)
            .map(|(i, _)| i)
            .collect();

        for i in index..index + count {
            if let Some(&char_idx) = visible.get(i) {
                let id = self.chars[char_idx].id;
                ops.push(TextOperation {
                    id: self.next_counter,
                    site: self.site,
                    op: CrdtOp::Delete(id),
                });
                self.apply_op(&CrdtOp::Delete(id));
            }
        }

        ops
    }

    /// Get the origin_left and origin_right for inserting at a visible index.
    fn get_origins_at(&self, index: usize) -> (CharId, CharId) {
        let visible: Vec<&CrdtChar> = self.chars.iter().filter(|c| !c.deleted).collect();
        let left = if index == 0 {
            CharId {
                site: 0,
                counter: 0,
            }
        } else if let Some(ch) = visible.get(index - 1) {
            ch.id
        } else {
            CharId {
                site: 0,
                counter: 0,
            }
        };

        let right = if let Some(ch) = visible.get(index) {
            ch.id
        } else if let Some(ch) = visible.last() {
            ch.id
        } else {
            CharId {
                site: 0,
                counter: 0,
            }
        };

        (left, right)
    }

    /// Apply a remote operation.
    pub fn apply_op(&mut self, op: &CrdtOp) {
        match op {
            CrdtOp::Insert(ch) => {
                let pos = self.find_insert_position(ch);
                self.chars.insert(pos, ch.clone());
            }
            CrdtOp::Delete(id) => {
                if let Some(ch) = self.chars.iter_mut().find(|c| c.id == *id) {
                    ch.deleted = true;
                }
            }
        }
    }

    /// Find the correct insertion position based on origin ordering.
    ///
    /// Characters with the same origin pair are ordered deterministically
    /// by their `CharId` (site, then counter) to ensure convergence.
    fn find_insert_position(&self, ch: &CrdtChar) -> usize {
        let mut pos = 0;
        let mut found_left = false;

        for (i, existing) in self.chars.iter().enumerate() {
            if existing.id == ch.origin_right {
                return pos;
            }
            if existing.id == ch.origin_left {
                found_left = true;
                pos = i + 1;
                continue;
            }
            if found_left
                && existing.origin_left == ch.origin_left
                && existing.origin_right == ch.origin_right
            {
                if existing.id > ch.id {
                    return i;
                }
                pos = i + 1;
            }
        }
        pos
    }

    /// Get the visible text content.
    pub fn text(&self) -> String {
        self.chars
            .iter()
            .filter(|c| !c.deleted)
            .map(|c| c.value)
            .collect()
    }

    /// Get the length of visible text.
    pub fn len(&self) -> usize {
        self.chars.iter().filter(|c| !c.deleted).count()
    }

    /// Check if document is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_document_is_empty() {
        let doc = CrdtDocument::new(1);
        assert!(doc.is_empty());
        assert_eq!(doc.len(), 0);
        assert_eq!(doc.text(), "");
    }

    #[test]
    fn insert_single_char() {
        let mut doc = CrdtDocument::new(1);
        let ops = doc.insert(0, "a");
        assert_eq!(ops.len(), 1);
        assert_eq!(doc.text(), "a");
        assert_eq!(doc.len(), 1);
    }

    #[test]
    fn insert_text() {
        let mut doc = CrdtDocument::new(1);
        doc.insert(0, "hello");
        assert_eq!(doc.text(), "hello");
        assert_eq!(doc.len(), 5);
    }

    #[test]
    fn delete_char() {
        let mut doc = CrdtDocument::new(1);
        doc.insert(0, "abc");
        let ops = doc.delete(1, 1);
        assert_eq!(ops.len(), 1);
        assert_eq!(doc.text(), "ac");
        assert_eq!(doc.len(), 2);
    }

    #[test]
    fn concurrent_insert() {
        let mut doc_a = CrdtDocument::new(1);
        let mut doc_b = CrdtDocument::new(2);

        let init_ops = doc_a.insert(0, "ab");
        for op in &init_ops {
            doc_b.apply_op(&op.op);
        }
        assert_eq!(doc_a.text(), "ab");
        assert_eq!(doc_b.text(), "ab");

        let ops_a = doc_a.insert(1, "X");
        let ops_b = doc_b.insert(1, "Y");

        for op in &ops_a {
            doc_b.apply_op(&op.op);
        }
        for op in &ops_b {
            doc_a.apply_op(&op.op);
        }

        let text_a = doc_a.text();
        let text_b = doc_b.text();

        assert_eq!(text_a, text_b, "both sites must converge to the same text");
        assert_eq!(text_a.len(), 4);
        assert!(text_a.contains('a'));
        assert!(text_a.contains('b'));
        assert!(text_a.contains('X'));
        assert!(text_a.contains('Y'));
    }

    #[test]
    fn concurrent_delete() {
        let mut doc_a = CrdtDocument::new(1);
        let mut doc_b = CrdtDocument::new(2);

        let init_ops = doc_a.insert(0, "abcd");
        for op in &init_ops {
            doc_b.apply_op(&op.op);
        }

        let ops_a = doc_a.delete(0, 1);
        let ops_b = doc_b.delete(3, 1);

        for op in &ops_a {
            doc_b.apply_op(&op.op);
        }
        for op in &ops_b {
            doc_a.apply_op(&op.op);
        }

        assert_eq!(doc_a.text(), doc_b.text());
        assert_eq!(doc_a.text(), "bc");
    }

    #[test]
    fn text_retrieval() {
        let mut doc = CrdtDocument::new(1);
        doc.insert(0, "hello world");
        assert_eq!(doc.text(), "hello world");

        doc.delete(5, 1);
        assert_eq!(doc.text(), "helloworld");

        doc.insert(5, " ");
        assert_eq!(doc.text(), "hello world");
    }

    #[test]
    fn delete_range() {
        let mut doc = CrdtDocument::new(1);
        doc.insert(0, "abcdef");
        let ops = doc.delete(2, 3);
        assert_eq!(ops.len(), 3);
        assert_eq!(doc.text(), "abf");
        assert_eq!(doc.len(), 3);
    }
}
