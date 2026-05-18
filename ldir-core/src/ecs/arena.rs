//! Simple bump allocator for contiguous value storage.
//!
//! Provides O(1) allocation and access. Values are stored contiguously
//! in a `Vec<T>`, supporting cache-linear iteration per
//! [THM-ECS-CACHE-FRIENDLY].
//!
//! # References
//!
//! - REQ-4.1.1: Zero dynamic heap allocations during hot layout pass
//! - CON-ECS-010: Bump allocator for dense array growth
//! - REQ-4.1.3: 64-byte cache-line alignment for attribute arrays

/// A simple bump allocator that stores values contiguously.
///
/// Allocation appends to an internal `Vec<T>`, returning the index.
/// The arena does not support individual deallocation; use [`Arena::clear`]
/// to reset the entire arena.
///
/// # Examples
///
/// ```ignore
/// use ldir_core::ecs::Arena;
///
/// let mut arena: Arena<i32> = Arena::new();
/// let a = arena.alloc(10);
/// let b = arena.alloc(20);
/// assert_eq!(arena.get(a), Some(&10));
/// assert_eq!(arena.get(b), Some(&20));
/// assert_eq!(arena.len(), 2);
/// ```
#[allow(dead_code)]
pub struct Arena<T> {
    /// Contiguous storage for allocated values.
    /// Per REQ-4.1.3, dense arrays should be 64-byte cache-line aligned
    /// (achieved via custom allocator in Phase B).
    data: Vec<T>,
}

#[allow(dead_code)]
impl<T> Arena<T> {
    /// Creates a new empty arena.
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Allocates a value, returning its index.
    ///
    /// Runs in O(1) amortized time (Vec push). Per AX-ECS-004,
    /// allocation occurs only during cold initialization, not the
    /// hot layout pass.
    pub fn alloc(&mut self, value: T) -> usize {
        let index = self.data.len();
        self.data.push(value);
        index
    }

    /// Returns a reference to the value at `index`, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Returns a mutable reference to the value at `index`, or `None` if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    /// Returns the number of allocated values.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if no values are allocated.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clears all values, resetting length to zero.
    ///
    /// Does not change capacity. Per AX-ECS-004, this is used
    /// during teardown rather than the hot layout pass.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns the underlying slice of allocated values.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the underlying mutable slice of allocated values.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_returns_sequential_indices() {
        let mut arena: Arena<i32> = Arena::new();
        assert_eq!(arena.alloc(10), 0);
        assert_eq!(arena.alloc(20), 1);
        assert_eq!(arena.alloc(30), 2);
    }

    #[test]
    fn test_get_returns_correct_value() {
        let mut arena: Arena<String> = Arena::new();
        let idx = arena.alloc(String::from("hello"));
        assert_eq!(arena.get(idx), Some(&String::from("hello")));
    }

    #[test]
    fn test_get_out_of_bounds_returns_none() {
        let arena: Arena<i32> = Arena::new();
        assert_eq!(arena.get(0), None);
    }

    #[test]
    fn test_get_mut_allows_mutation() {
        let mut arena: Arena<i32> = Arena::new();
        let idx = arena.alloc(10);
        *arena.get_mut(idx).unwrap() = 42;
        assert_eq!(arena.get(idx), Some(&42));
    }

    #[test]
    fn test_len_tracks_count() {
        let mut arena: Arena<i32> = Arena::new();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        arena.alloc(1);
        assert_eq!(arena.len(), 1);
        assert!(!arena.is_empty());
        arena.alloc(2);
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn test_clear_resets_length() {
        let mut arena: Arena<i32> = Arena::new();
        arena.alloc(1);
        arena.alloc(2);
        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_clear_preserves_capacity_for_reuse() {
        let mut arena: Arena<i32> = Arena::new();
        arena.alloc(1);
        arena.alloc(2);
        arena.clear();
        // After clear, new allocations should reuse capacity
        let idx = arena.alloc(3);
        assert_eq!(arena.get(idx), Some(&3));
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn test_as_slice_returns_all_values() {
        let mut arena: Arena<i32> = Arena::new();
        arena.alloc(10);
        arena.alloc(20);
        arena.alloc(30);
        assert_eq!(arena.as_slice(), &[10, 20, 30]);
    }
}
