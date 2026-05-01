//! Sparse set and typed component storage for the ECS.
//!
//! [`SparseSet`] provides O(1) entity-to-index mapping via double
//! indirection (DEF-SPARSE-SET). [`ComponentStore<T>`] uses an SoA
//! layout with a dense `Vec<T>` for component data and a sparse set
//! for entity lookup.
//!
//! # References
//!
//! - DEF-SPARSE-SET: O(1) entity-to-component mapping
//! - THM-ECS-ACCESS: Component access is O(1) via double indirection
//! - LEM-ECS-003: Swap-and-pop preserves dense compactness
//! - REQ-4.1.2: Structure of Arrays layout for all node attributes
//! - REQ-4.1.4: No raw pointers or Box/Rc/Arc; all relations via 32-bit indices

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::ecs::entity::Entity;

/// Component type identifier. Supports up to 256 component types.
///
/// Per NC-ECS-007: max component size is 64 bytes (fits one cache line).
pub type ComponentId = u8;

/// A sparse set mapping keys to dense array indices.
///
/// Provides O(1) insert, remove, and lookup via a `HashMap` for the
/// sparse array and a `Vec` for the dense array. Removal uses
/// swap-and-pop to maintain compactness (LEM-ECS-003).
///
/// The dense array is kept parallel to an external data array
/// (e.g., `ComponentStore::data`).
///
/// # References
///
/// - DEF-SPARSE-SET: s = (sparse, dense, entities)
/// - THM-ECS-ACCESS: O(1) component access
/// - LEM-ECS-003: Swap-and-pop preserves dense compactness
pub struct SparseSet<K> {
    /// Sparse array: key → dense index. O(1) lookup.
    sparse: HashMap<K, usize>,
    /// Dense array of keys, parallel to external data arrays.
    /// Maintained compact (gap-free) via swap-and-pop.
    dense: Vec<K>,
}

impl<K> SparseSet<K>
where
    K: Eq + std::hash::Hash + Clone,
{
    /// Creates a new empty sparse set.
    pub fn new() -> Self {
        Self {
            sparse: HashMap::new(),
            dense: Vec::new(),
        }
    }

    /// Inserts a key, returning `(dense_index, is_new)`.
    ///
    /// If the key already exists, returns its current dense index
    /// with `is_new = false` (no-op). Otherwise, appends to the
    /// dense array and returns the new index with `is_new = true`.
    pub fn insert(&mut self, key: K) -> (usize, bool) {
        let dense_idx = self.dense.len();
        match self.sparse.entry(key.clone()) {
            Entry::Occupied(entry) => (*entry.get(), false),
            Entry::Vacant(entry) => {
                entry.insert(dense_idx);
                self.dense.push(key);
                (dense_idx, true)
            }
        }
    }

    /// Removes a key via swap-and-pop, returning its dense index.
    ///
    /// Per LEM-ECS-003: maintains the dense array as a compact,
    /// contiguous block with no gaps.
    ///
    /// Returns `None` if the key is not present.
    pub fn remove(&mut self, key: &K) -> Option<usize> {
        let dense_idx = *self.sparse.get(key)?;
        self.sparse.remove(key);

        let last_idx = self.dense.len() - 1;
        if dense_idx != last_idx {
            // Swap with last element and update its sparse entry
            let swapped_key = self.dense[last_idx].clone();
            self.dense.swap(dense_idx, last_idx);
            self.sparse.insert(swapped_key, dense_idx);
        }
        self.dense.pop();
        Some(dense_idx)
    }

    /// Returns the dense index for `key`, or `None` if not present.
    ///
    /// Per THM-ECS-ACCESS: O(1) via HashMap lookup.
    pub fn get(&self, key: &K) -> Option<usize> {
        self.sparse.get(key).copied()
    }

    /// Returns `true` if the key is present in the set.
    pub fn contains(&self, key: &K) -> bool {
        self.sparse.contains_key(key)
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    /// Returns `true` if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    /// Returns the dense key array.
    ///
    /// The dense array is kept parallel to the external data array
    /// (e.g., `ComponentStore::data`). Dense index `i` in this array
    /// corresponds to dense index `i` in the data array.
    pub fn dense(&self) -> &[K] {
        &self.dense
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.sparse.clear();
        self.dense.clear();
    }
}

impl<K> Default for SparseSet<K>
where
    K: Eq + std::hash::Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Typed storage for a single component type.
///
/// Uses Structure of Arrays (SoA) layout per REQ-4.1.2:
/// - `data`: contiguous `Vec<T>` for component data
/// - `sparse`: [`SparseSet<Entity>`] for entity-to-index mapping
///
/// Access is O(1) via double indirection (THM-ECS-ACCESS).
/// Dense arrays are compact and cache-friendly (THM-ECS-CACHE-FRIENDLY).
///
/// # References
///
/// - DEF-STORAGE: Component storage specification
/// - REQ-4.1.2: SoA layout for all node attributes
/// - REQ-4.1.4: No Box/Rc/Arc for document nodes
/// - THM-ECS-ACCESS: O(1) component access
/// - THM-ECS-CACHE-FRIENDLY: Cache-linear single-component iteration
pub struct ComponentStore<T> {
    /// Dense array of component data. Per REQ-4.1.3, should be
    /// 64-byte cache-line aligned (custom allocator in Phase B).
    data: Vec<T>,
    /// Sparse set mapping Entity → dense index.
    /// Parallel to `data`: dense index `i` in the sparse set
    /// corresponds to `data[i]`.
    sparse: SparseSet<Entity>,
}

impl<T> ComponentStore<T> {
    /// Creates a new empty component store.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            sparse: SparseSet::new(),
        }
    }

    /// Inserts a component for the given entity.
    ///
    /// If the entity already has this component, the value is updated
    /// in place (no new dense slot allocated).
    ///
    /// Per ALG-ECS-CREATE lines 24-33.
    pub fn insert(&mut self, entity: Entity, component: T) {
        let (dense_idx, is_new) = self.sparse.insert(entity);
        if is_new {
            self.data.push(component);
        } else {
            self.data[dense_idx] = component;
        }
    }

    /// Removes the component for the given entity via swap-and-pop.
    ///
    /// Per LEM-ECS-003: maintains dense array compactness.
    /// Per ALG-ECS-DESTROY: O(1) removal.
    ///
    /// Returns the removed component, or `None` if the entity
    /// does not have this component.
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let dense_idx = self.sparse.remove(&entity)?;
        Some(self.data.swap_remove(dense_idx))
    }

    /// Returns a reference to the component for the given entity.
    ///
    /// Per THM-ECS-ACCESS: O(1) via double indirection.
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let dense_idx = self.sparse.get(&entity)?;
        self.data.get(dense_idx)
    }

    /// Returns a mutable reference to the component for the given entity.
    ///
    /// Per THM-ECS-ACCESS: O(1) via double indirection.
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let dense_idx = self.sparse.get(&entity)?;
        self.data.get_mut(dense_idx)
    }

    /// Returns `true` if the entity has this component.
    pub fn contains(&self, entity: Entity) -> bool {
        self.sparse.contains(&entity)
    }

    /// Iterates over all (Entity, &T) pairs.
    ///
    /// Returns entries in dense array order (insertion order with
    /// swap-and-pop reordering on removal). For deterministic
    /// ascending entity ID order, see [`World::query`](super::storage::World::query).
    ///
    /// Per THM-ECS-CACHE-FRIENDLY: touches memory in a strictly
    /// linear pattern for maximum cache line utilization.
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> + '_ {
        self.sparse.dense().iter().copied().zip(self.data.iter())
    }

    /// Returns the number of entities with this component.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if no entities have this component.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clears all components.
    pub fn clear(&mut self) {
        self.data.clear();
        self.sparse.clear();
    }
}

impl<T> Default for ComponentStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::entity::Entity;

    // === SparseSet tests ===

    #[test]
    fn test_sparse_set_insert_and_get() {
        let mut set = SparseSet::new();
        let (idx, is_new) = set.insert(42u32);
        assert_eq!(idx, 0);
        assert!(is_new);
        assert_eq!(set.get(&42), Some(0));
        assert!(set.contains(&42));
    }

    #[test]
    fn test_sparse_set_insert_duplicate_returns_existing() {
        let mut set = SparseSet::new();
        set.insert(42u32);
        let (idx, is_new) = set.insert(42u32);
        assert_eq!(idx, 0);
        assert!(!is_new);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_sparse_set_multiple_inserts() {
        let mut set = SparseSet::new();
        set.insert(10u32);
        set.insert(20u32);
        set.insert(30u32);
        assert_eq!(set.get(&10), Some(0));
        assert_eq!(set.get(&20), Some(1));
        assert_eq!(set.get(&30), Some(2));
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_sparse_set_remove_existing() {
        let mut set = SparseSet::new();
        set.insert(10u32);
        set.insert(20u32);
        set.insert(30u32);
        let idx = set.remove(&20).unwrap();
        assert_eq!(idx, 1);
        assert!(!set.contains(&20));
        assert_eq!(set.len(), 2);
        // Verify dense compactness: no gaps
        assert_eq!(set.dense().len(), 2);
    }

    #[test]
    fn test_sparse_set_remove_nonexistent_returns_none() {
        let mut set: SparseSet<u32> = SparseSet::new();
        assert_eq!(set.remove(&42), None);
    }

    #[test]
    fn test_sparse_set_swap_and_pop_updates_swapped_key() {
        let mut set = SparseSet::new();
        set.insert(10u32); // dense[0]
        set.insert(20u32); // dense[1]
        set.insert(30u32); // dense[2]

        // Remove middle element (dense index 1)
        set.remove(&20);
        // The last element (30) should have moved to index 1
        assert_eq!(set.get(&30), Some(1));
        assert_eq!(set.get(&10), Some(0));
        // Dense array should be [10, 30] (compact, no gaps per LEM-ECS-003)
        assert_eq!(set.dense(), &[10, 30]);
    }

    #[test]
    fn test_sparse_set_remove_last_element() {
        let mut set = SparseSet::new();
        set.insert(10u32);
        set.insert(20u32);
        let idx = set.remove(&20).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(set.dense(), &[10]);
        assert_eq!(set.get(&10), Some(0));
    }

    #[test]
    fn test_sparse_set_remove_only_element() {
        let mut set = SparseSet::new();
        set.insert(42u32);
        set.remove(&42);
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_sparse_set_dense_compactness_after_many_removals() {
        let mut set = SparseSet::new();
        for i in 0..100u32 {
            set.insert(i);
        }
        // Remove every other element
        for i in (0..100).step_by(2) {
            set.remove(&i);
        }
        // All remaining should have valid indices
        for i in (1..100).step_by(2) {
            let idx = set.get(&i).unwrap();
            assert!(idx < set.len());
        }
        // Dense array should be compact
        assert_eq!(set.len(), set.dense().len());
    }

    // === ComponentStore tests ===

    fn make_entity(index: u32) -> Entity {
        Entity::new(index, 0)
    }

    #[test]
    fn test_component_insert_and_get() {
        let mut store = ComponentStore::<i32>::new();
        let e = make_entity(0);
        store.insert(e, 42);
        assert_eq!(store.get(e), Some(&42));
        assert!(store.contains(e));
    }

    #[test]
    fn test_component_get_nonexistent_returns_none() {
        let store: ComponentStore<i32> = ComponentStore::new();
        assert_eq!(store.get(make_entity(0)), None);
    }

    #[test]
    fn test_component_insert_multiple_entities() {
        let mut store = ComponentStore::<i32>::new();
        let e0 = make_entity(0);
        let e1 = make_entity(1);
        let e2 = make_entity(2);
        store.insert(e0, 10);
        store.insert(e1, 20);
        store.insert(e2, 30);
        assert_eq!(store.get(e0), Some(&10));
        assert_eq!(store.get(e1), Some(&20));
        assert_eq!(store.get(e2), Some(&30));
        assert_eq!(store.len(), 3);
    }

    #[test]
    fn test_component_update_existing() {
        let mut store = ComponentStore::<i32>::new();
        let e = make_entity(0);
        store.insert(e, 10);
        store.insert(e, 99); // update
        assert_eq!(store.get(e), Some(&99));
        assert_eq!(store.len(), 1); // no new slot
    }

    #[test]
    fn test_component_remove() {
        let mut store = ComponentStore::<i32>::new();
        let e0 = make_entity(0);
        let e1 = make_entity(1);
        let e2 = make_entity(2);
        store.insert(e0, 10);
        store.insert(e1, 20);
        store.insert(e2, 30);
        let removed = store.remove(e1).unwrap();
        assert_eq!(removed, 20);
        assert_eq!(store.get(e1), None);
        assert_eq!(store.len(), 2);
        // Remaining should still be accessible
        assert!(store.contains(e0));
        assert!(store.contains(e2));
    }

    #[test]
    fn test_component_remove_nonexistent_returns_none() {
        let mut store: ComponentStore<i32> = ComponentStore::new();
        assert_eq!(store.remove(make_entity(0)), None);
    }

    #[test]
    fn test_component_get_mut() {
        let mut store = ComponentStore::<i32>::new();
        let e = make_entity(0);
        store.insert(e, 10);
        *store.get_mut(e).unwrap() = 99;
        assert_eq!(store.get(e), Some(&99));
    }

    #[test]
    fn test_component_iter_returns_all() {
        let mut store = ComponentStore::<i32>::new();
        let e0 = make_entity(0);
        let e1 = make_entity(1);
        let e2 = make_entity(2);
        store.insert(e0, 10);
        store.insert(e1, 20);
        store.insert(e2, 30);
        let items: std::collections::HashSet<_> = store.iter().collect();
        assert!(items.contains(&(e0, &10)));
        assert!(items.contains(&(e1, &20)));
        assert!(items.contains(&(e2, &30)));
    }

    #[test]
    fn test_component_iter_empty() {
        let store: ComponentStore<i32> = ComponentStore::new();
        assert_eq!(store.iter().count(), 0);
    }

    #[test]
    fn test_component_remove_middle_preserves_access() {
        let mut store = ComponentStore::<i32>::new();
        let e0 = make_entity(0);
        let e1 = make_entity(1);
        let e2 = make_entity(2);
        store.insert(e0, 10);
        store.insert(e1, 20);
        store.insert(e2, 30);
        store.remove(e1);
        // After swap-and-pop, e2 moved to dense index 1
        assert_eq!(store.get(e2), Some(&30));
        assert_eq!(store.get(e0), Some(&10));
    }

    #[test]
    fn test_component_with_struct_data() {
        #[derive(Debug, Clone, PartialEq)]
        struct Position {
            x: f64,
            y: f64,
        }
        let mut store = ComponentStore::<Position>::new();
        let e = make_entity(5);
        store.insert(e, Position { x: 1.0, y: 2.0 });
        assert_eq!(store.get(e), Some(&Position { x: 1.0, y: 2.0 }));
        let pos = store.get_mut(e).unwrap();
        pos.x = 10.0;
        assert_eq!(store.get(e).unwrap().x, 10.0);
    }

    #[test]
    fn test_component_clear() {
        let mut store = ComponentStore::<i32>::new();
        store.insert(make_entity(0), 10);
        store.insert(make_entity(1), 20);
        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }
}
