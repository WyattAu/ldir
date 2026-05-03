//! String interner for deduplicating repeated text across the compilation pipeline.

#![deny(unsafe_code)]

use std::collections::HashMap;

/// A string interner that deduplicates strings.
///
/// Returns a unique `u32` ID for each distinct string. Repeated calls with
/// the same text return the same ID, avoiding duplicate allocations.
pub struct StringInterner {
    strings: HashMap<String, u32>,
    values: Vec<String>,
    total_unique_bytes: usize,
    duplicate_bytes_saved: usize,
}

impl StringInterner {
    /// Create a new empty interner.
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            values: Vec::new(),
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
        let id = self.values.len() as u32;
        self.total_unique_bytes += s.len();
        self.strings.insert(s.to_string(), id);
        self.values.push(s.to_string());
        id
    }

    /// Get the string for an ID.
    ///
    /// Returns `None` if the ID is out of range.
    pub fn get(&self, id: u32) -> Option<&str> {
        self.values.get(id as usize).map(|s| s.as_str())
    }

    /// Number of interned strings.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
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
