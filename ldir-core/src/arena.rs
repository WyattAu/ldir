//! Typed arena allocator and string arena for compilation hot paths.
//!
//! Provides O(1) bump allocation. No individual deallocation;
//! drop the arena (or call [`crate::arena::Arena::clear`]) to free all memory at once.
//!
//! # Design
//!
//! Uses a chunked `Vec<Vec<T>>` backing store to avoid reallocation
//! of the entire arena when a single chunk fills up. Items within
//! a chunk are contiguous and cache-friendly.

/// A typed bump allocator that stores `T` values in fixed-size chunks.
///
/// Allocation is O(1) amortized. No individual deallocation.
/// Drop the arena (or call [`Self::clear`]) to free everything.
///
/// Returns indices for allocated items. Access via [`get`][`Arena::get`]
/// or [`get_mut`][`Arena::get_mut`].
pub struct Arena<T> {
    chunks: Vec<Vec<T>>,
    current: Vec<T>,
    len: usize,
    chunk_size: usize,
    #[cfg(debug_assertions)]
    alloc_count: usize,
}

impl<T> Arena<T> {
    /// Create a new arena with the given chunk capacity.
    pub fn with_capacity(chunk_size: usize) -> Self {
        let chunk_size = chunk_size.max(1);
        Self {
            chunks: Vec::new(),
            current: Vec::with_capacity(chunk_size),
            len: 0,
            chunk_size,
            #[cfg(debug_assertions)]
            alloc_count: 0,
        }
    }

    /// Create a new arena with default chunk capacity (256).
    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    /// Allocate a value, returning its index.
    ///
    /// Runs in O(1) amortized time.
    pub fn alloc(&mut self, value: T) -> usize {
        if self.current.len() >= self.current.capacity() {
            self.push_new_chunk();
        }
        let index = self.len;
        self.current.push(value);
        self.len += 1;
        #[cfg(debug_assertions)]
        {
            self.alloc_count += 1;
        }
        index
    }

    /// Allocate multiple values contiguously.
    ///
    /// Returns the starting index. Items are stored at indices
    /// `[start, start + values.len())`.
    pub fn alloc_batch(&mut self, values: &[T]) -> usize
    where
        T: Clone,
    {
        if values.is_empty() {
            return self.len;
        }
        if values.len() > self.chunk_size {
            self.chunk_size = values.len().next_power_of_two();
        }
        if self.current.len() + values.len() > self.current.capacity() {
            self.push_new_chunk();
        }
        let start = self.len;
        self.current.extend_from_slice(values);
        self.len += values.len();
        #[cfg(debug_assertions)]
        {
            self.alloc_count += values.len();
        }
        start
    }

    /// Get a reference to the value at `index`.
    ///
    /// Returns `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            self.resolve(index)
        } else {
            None
        }
    }

    /// Get a mutable reference to the value at `index`.
    ///
    /// Returns `None` if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            self.resolve_mut(index)
        } else {
            None
        }
    }

    /// Number of allocated items.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Reset the arena without deallocating memory.
    ///
    /// Clears all items. Chunks remain allocated for reuse.
    pub fn clear(&mut self) {
        self.current.clear();
        for chunk in &mut self.chunks {
            chunk.clear();
        }
        self.chunks.clear();
        self.len = 0;
    }

    /// Total bytes consumed by allocated items.
    pub fn used_bytes(&self) -> usize {
        let item_size = std::mem::size_of::<T>();
        let full_count: usize = self.chunks.iter().map(|c| c.len()).sum();
        (full_count + self.current.len()) * item_size
    }

    /// Number of individual allocations performed (debug builds only).
    #[cfg(debug_assertions)]
    pub fn alloc_count(&self) -> usize {
        self.alloc_count
    }

    /// Number of individual allocations performed (always 0 in release).
    #[cfg(not(debug_assertions))]
    pub fn alloc_count(&self) -> usize {
        0
    }

    fn resolve(&self, index: usize) -> Option<&T> {
        let chunk_capacity = self.chunk_size;
        let chunk_idx = index / chunk_capacity;
        let offset = index % chunk_capacity;
        if chunk_idx < self.chunks.len() {
            self.chunks.get(chunk_idx).and_then(|c| c.get(offset))
        } else if chunk_idx == self.chunks.len() {
            self.current.get(offset)
        } else {
            None
        }
    }

    fn resolve_mut(&mut self, index: usize) -> Option<&mut T> {
        let chunk_capacity = self.chunk_size;
        let chunk_idx = index / chunk_capacity;
        let offset = index % chunk_capacity;
        if chunk_idx < self.chunks.len() {
            self.chunks
                .get_mut(chunk_idx)
                .and_then(|c| c.get_mut(offset))
        } else if chunk_idx == self.chunks.len() {
            self.current.get_mut(offset)
        } else {
            None
        }
    }

    fn push_new_chunk(&mut self) {
        let mut new_chunk = Vec::with_capacity(self.chunk_size);
        std::mem::swap(&mut new_chunk, &mut self.current);
        if !new_chunk.is_empty() {
            self.chunks.push(new_chunk);
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::with_capacity(256)
    }
}

/// Arena for storing string data contiguously.
///
/// Returns indices for allocated strings. Access via [`get`][`StringArena::get`].
/// Ideal for deduplicating string allocations during compilation.
pub struct StringArena {
    data: Vec<String>,
}

impl StringArena {
    /// Create a new empty string arena.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Allocate a string, returning its index.
    pub fn alloc(&mut self, s: &str) -> usize {
        let index = self.data.len();
        self.data.push(s.to_string());
        index
    }

    /// Get the string at `index`.
    ///
    /// Returns `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&str> {
        self.data.get(index).map(|s| s.as_str())
    }

    /// Total bytes consumed by allocated string data.
    pub fn used_bytes(&self) -> usize {
        self.data.iter().map(|s| s.len()).sum()
    }

    /// Clear the arena without deallocating.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Number of allocated strings.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for StringArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic_alloc() {
        let mut arena: Arena<i32> = Arena::with_capacity(4);
        let a = arena.alloc(10);
        let b = arena.alloc(20);
        let c = arena.alloc(30);
        assert_eq!(arena.get(a), Some(&10));
        assert_eq!(arena.get(b), Some(&20));
        assert_eq!(arena.get(c), Some(&30));
        assert_eq!(arena.len(), 3);
    }

    #[test]
    fn test_arena_batch_alloc() {
        let mut arena: Arena<i32> = Arena::with_capacity(8);
        let start = arena.alloc_batch(&[1, 2, 3, 4]);
        assert_eq!(start, 0);
        assert_eq!(arena.get(start), Some(&1));
        assert_eq!(arena.get(start + 1), Some(&2));
        assert_eq!(arena.get(start + 2), Some(&3));
        assert_eq!(arena.get(start + 3), Some(&4));
        assert_eq!(arena.len(), 4);
    }

    #[test]
    fn test_arena_multiple_chunks() {
        let mut arena: Arena<i32> = Arena::with_capacity(2);
        let mut indices = Vec::new();
        for i in 0..10 {
            let idx = arena.alloc(i);
            indices.push(idx);
        }
        assert_eq!(arena.len(), 10);
        for (expected, &idx) in indices.iter().enumerate() {
            assert_eq!(arena.get(idx), Some(&(expected as i32)));
        }
        assert_eq!(arena.used_bytes(), 10 * std::mem::size_of::<i32>());
    }

    #[test]
    fn test_arena_clear_reuses_memory() {
        let mut arena: Arena<String> = Arena::with_capacity(4);
        arena.alloc("hello".to_string());
        arena.alloc("world".to_string());
        assert_eq!(arena.len(), 2);
        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        let idx = arena.alloc("reused".to_string());
        assert_eq!(arena.get(idx), Some(&"reused".to_string()));
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn test_arena_used_bytes() {
        let mut arena: Arena<u64> = Arena::with_capacity(4);
        arena.alloc(1);
        arena.alloc(2);
        arena.alloc(3);
        assert_eq!(arena.used_bytes(), 3 * 8);
    }

    #[test]
    fn test_arena_alloc_count_debug() {
        let mut arena: Arena<i32> = Arena::with_capacity(4);
        arena.alloc(1);
        arena.alloc(2);
        arena.alloc_batch(&[3, 4]);
        assert_eq!(arena.alloc_count(), 4);
    }

    #[test]
    fn test_arena_default() {
        let arena: Arena<i32> = Arena::default();
        assert!(arena.is_empty());
        assert_eq!(arena.chunk_size, 256);
    }

    #[test]
    fn test_arena_empty_batch() {
        let mut arena: Arena<i32> = Arena::with_capacity(4);
        let start = arena.alloc_batch(&[]);
        assert_eq!(start, 0);
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn test_arena_get_mut() {
        let mut arena: Arena<i32> = Arena::with_capacity(4);
        let idx = arena.alloc(10);
        *arena.get_mut(idx).unwrap() = 42;
        assert_eq!(arena.get(idx), Some(&42));
    }

    #[test]
    fn test_arena_get_out_of_bounds() {
        let arena: Arena<i32> = Arena::new();
        assert_eq!(arena.get(0), None);
    }

    #[test]
    fn test_string_arena_basic() {
        let mut arena = StringArena::new();
        let i1 = arena.alloc("hello");
        let i2 = arena.alloc("world");
        assert_eq!(arena.get(i1), Some("hello"));
        assert_eq!(arena.get(i2), Some("world"));
    }

    #[test]
    fn test_string_arena_many_strings() {
        let mut arena = StringArena::new();
        for i in 0..100 {
            let s = format!("str_{}", i);
            let idx = arena.alloc(&s);
            assert_eq!(arena.get(idx), Some(s.as_str()));
        }
        assert_eq!(arena.len(), 100);
    }

    #[test]
    fn test_string_arena_clear() {
        let mut arena = StringArena::new();
        arena.alloc("hello");
        arena.alloc("world");
        assert_eq!(arena.len(), 2);
        assert!(!arena.is_empty());
        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        let idx = arena.alloc("reused");
        assert_eq!(arena.get(idx), Some("reused"));
    }

    #[test]
    fn test_string_arena_used_bytes() {
        let mut arena = StringArena::new();
        arena.alloc("hello");
        arena.alloc("world");
        assert_eq!(arena.used_bytes(), 10);
    }
}
