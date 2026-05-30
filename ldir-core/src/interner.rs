//! String interner for deduplicating repeated text across the compilation pipeline.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

/// A string interner that deduplicates strings.
///
/// Returns a unique `u32` ID for each distinct string. Repeated calls with
/// the same text return the same ID, avoiding duplicate allocations.
///
/// Uses `Arc<str>` so that both the `Vec` and `HashMap` share a single
/// heap allocation per unique string. Cloning an `Arc` only bumps the
/// reference count.
#[derive(Debug, Clone)]
pub struct StringInterner {
    /// Maps string content to its assigned ID.
    strings: HashMap<Arc<str>, u32>,
    /// Reverse mapping: ID -> string content, enabling O(1) `get()`.
    by_id: Vec<Arc<str>>,
    total_unique_bytes: usize,
    duplicate_bytes_saved: usize,
}

impl StringInterner {
    /// Create a new empty interner.
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            by_id: Vec::new(),
            total_unique_bytes: 0,
            duplicate_bytes_saved: 0,
        }
    }

    /// Intern a string, returning a unique ID.
    ///
    /// If the string was already interned, returns the existing ID.
    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.strings.get(s) {
            self.duplicate_bytes_saved += s.len();
            return id;
        }
        let id = self.by_id.len() as u32;
        self.total_unique_bytes += s.len();
        let shared: Arc<str> = s.into();
        self.by_id.push(shared.clone());
        self.strings.insert(shared, id);
        id
    }

    /// Get the string for an ID.
    ///
    /// Returns `None` if the ID is out of range. O(1) lookup.
    pub fn get(&self, id: u32) -> Option<&str> {
        self.by_id.get(id as usize).map(|s| &**s)
    }

    /// Number of interned strings.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Check if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Total bytes of all unique strings stored.
    pub fn total_bytes(&self) -> usize {
        self.total_unique_bytes
    }

    /// Total bytes saved by deduplication.
    ///
    /// Counts bytes for every `intern()` call that returned an existing ID,
    /// i.e., `sum(len(s) for s that was already interned)`.
    pub fn bytes_saved(&self) -> usize {
        self.duplicate_bytes_saved
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interner_new_is_empty() {
        let interner = StringInterner::new();
        assert!(interner.is_empty());
        assert_eq!(interner.len(), 0);
    }

    #[test]
    fn intern_string_returns_id() {
        let mut interner = StringInterner::new();
        let id = interner.intern("hello");
        assert_eq!(id, 0);
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn intern_same_string_same_id() {
        let mut interner = StringInterner::new();
        let id1 = interner.intern("hello");
        let id2 = interner.intern("hello");
        assert_eq!(id1, id2);
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn intern_different_strings_different_ids() {
        let mut interner = StringInterner::new();
        let id1 = interner.intern("hello");
        let id2 = interner.intern("world");
        assert_ne!(id1, id2);
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn get_returns_original_string() {
        let mut interner = StringInterner::new();
        let id = interner.intern("hello world");
        assert_eq!(interner.get(id), Some("hello world"));
    }

    #[test]
    fn bytes_saved_counts() {
        let mut interner = StringInterner::new();
        interner.intern("hello"); // unique: 5 bytes
        interner.intern("hello"); // duplicate: saves 5 bytes
        interner.intern("world"); // unique: 5 bytes
        interner.intern("hello"); // duplicate: saves 5 bytes
        assert_eq!(interner.bytes_saved(), 10);
        assert_eq!(interner.len(), 2);
        assert_eq!(interner.total_bytes(), 10);
    }
}
