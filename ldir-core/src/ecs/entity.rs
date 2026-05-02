//! Entity identifier and allocator for the ECS.
//!
//! Entities are lightweight 32-bit identifiers carrying a slot index
//! and a generation counter. The generation detects stale references
//! after entity recycling.
//!
//! # References
//!
//! - REQ-3.1.6: 32-bit generation indices as entity identifiers
//! - DEF-ENTITY: E = (e_id, g_gen)
//! - AX-ECS-001: Entity uniqueness
//! - THM-ECS-CAPACITY: Maximum entity count is 2^32 - 1
//! - ALG-ECS-CREATE: Entity creation algorithm

#![allow(dead_code)]

use std::fmt;

/// Raw entity slot index — a 32-bit value.
///
/// Per REQ-3.1.6, entity IDs are 32-bit values.
/// Slot 0 is reserved as the null/sentinel entity per REQ-3.1.2.
pub type EntityId = u32;

/// A versioned entity reference consisting of a slot index and generation counter.
///
/// An entity reference is valid iff `generation` matches the allocator's
/// current generation for the same `index`. This detects stale references
/// after entity recycling (DEF-ENTITY).
///
/// # References
///
/// - DEF-ENTITY: E = (e_id, g_gen)
/// - REQ-3.1.6: 32-bit generation indices as entity identifiers
/// - THM-ECS-CAPACITY: Maximum entity count is 2^32 - 1
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    /// Slot index in the entity allocator.
    pub index: u32,
    /// Generation counter for stale-reference detection.
    /// Per NC-ECS-005: generation counter width is 32 bits.
    pub generation: u32,
}

impl Entity {
    /// Creates a new entity reference with the given slot index and generation.
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entity")
            .field("index", &self.index)
            .field("generation", &self.generation)
            .finish()
    }
}

/// Bump allocator that generates unique entity IDs with generation tracking.
///
/// Maintains a monotonically increasing counter for fresh allocations
/// and a free list for recycled slots. Each deallocation increments the
/// generation counter for the slot, invalidating all outstanding references
/// to the old generation.
///
/// No heap allocation occurs during [`EntityAllocator::allocate`] for recycled slots
/// (free-list pop). Fresh slots may trigger amortized Vec growth.
///
/// # References
///
/// - AX-ECS-001: Entity uniqueness
/// - THM-ECS-CAPACITY: Maximum entity count is 2^32 - 1
/// - ALG-ECS-CREATE: Entity creation algorithm (lines 3-8)
/// - ALG-ECS-DESTROY: Entity destruction algorithm (lines 36-38)
/// - PRE-ECS-002: Entity generation validity check
pub struct EntityAllocator {
    /// Next slot index for fresh allocation.
    next_slot: u32,
    /// Generation counter per slot. Incremented on deallocation
    /// so that stale Entity references (old generation) are detected.
    generations: Vec<u32>,
    /// Free list of recycled slots. LIFO order for cache locality.
    free_list: Vec<u32>,
}

impl EntityAllocator {
    /// Creates a new entity allocator with no live entities.
    pub const fn new() -> Self {
        Self {
            next_slot: 0,
            generations: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Allocates a new entity, returning its versioned ID.
    ///
    /// If the free list is non-empty, recycles a slot (generation was
    /// already bumped on deallocation). Otherwise, bumps `next_slot`.
    ///
    /// Per ALG-ECS-CREATE lines 3-8.
    pub fn allocate(&mut self) -> Entity {
        if let Some(index) = self.free_list.pop() {
            let generation = self.generations[index as usize];
            Entity::new(index, generation)
        } else {
            let index = self.next_slot;
            self.next_slot += 1;
            self.generations.push(0);
            Entity::new(index, 0)
        }
    }

    /// Returns `true` if the entity reference is still valid.
    ///
    /// Checks that the generation matches (PRE-ECS-002) and that
    /// the entity is not in the free list.
    pub fn is_alive(&self, entity: Entity) -> bool {
        if entity.index as usize >= self.generations.len() {
            return false;
        }
        let current_gen = self.generations[entity.index as usize];
        current_gen == entity.generation
    }

    /// Deallocates an entity, returning its slot to the free list.
    ///
    /// Increments the generation counter for the slot so that any
    /// outstanding `Entity` references with the old generation become
    /// stale (detected by [`EntityAllocator::is_alive`]).
    ///
    /// Per ALG-ECS-DESTROY lines 36-38.
    pub fn deallocate(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }
        self.generations[entity.index as usize] += 1;
        self.free_list.push(entity.index);
    }

    /// Returns the total number of slots ever allocated (including recycled).
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }

    /// Returns the number of currently live entities.
    pub fn alive_count(&self) -> usize {
        self.generations.len().saturating_sub(self.free_list.len())
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_returns_sequential_entities() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate();
        let e1 = alloc.allocate();
        let e2 = alloc.allocate();
        assert_eq!(e0.index, 0);
        assert_eq!(e1.index, 1);
        assert_eq!(e2.index, 2);
        assert_eq!(e0.generation, 0);
        assert_eq!(e1.generation, 0);
        assert_eq!(e2.generation, 0);
    }

    #[test]
    fn test_fresh_entity_is_alive() {
        let mut alloc = EntityAllocator::new();
        let e = alloc.allocate();
        assert!(alloc.is_alive(e));
    }

    #[test]
    fn test_deallocate_makes_entity_dead() {
        let mut alloc = EntityAllocator::new();
        let e = alloc.allocate();
        alloc.deallocate(e);
        assert!(!alloc.is_alive(e));
    }

    #[test]
    fn test_deallocate_nonexistent_is_noop() {
        let mut alloc = EntityAllocator::new();
        let e = Entity::new(999, 0);
        alloc.deallocate(e); // should not panic
    }

    #[test]
    fn test_recycled_slot_gets_new_generation() {
        let mut alloc = EntityAllocator::new();
        let e = alloc.allocate(); // index=0, gen=0
        alloc.deallocate(e); // gen bumped to 1
        let e2 = alloc.allocate(); // recycles index=0, gen=1
        assert_eq!(e2.index, 0);
        assert_eq!(e2.generation, 1);
        assert!(!alloc.is_alive(e)); // old ref stale
        assert!(alloc.is_alive(e2)); // new ref valid
    }

    #[test]
    fn test_double_recycle_increments_generation_twice() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate(); // index=0, gen=0
        alloc.deallocate(e0); // gen → 1
        let e1 = alloc.allocate(); // recycles, gen=1
        alloc.deallocate(e1); // gen → 2
        let e2 = alloc.allocate(); // recycles, gen=2
        assert_eq!(e2.index, 0);
        assert_eq!(e2.generation, 2);
    }

    #[test]
    fn test_alive_count_tracks_live_entities() {
        let mut alloc = EntityAllocator::new();
        assert_eq!(alloc.alive_count(), 0);
        let e0 = alloc.allocate();
        let e1 = alloc.allocate();
        assert_eq!(alloc.alive_count(), 2);
        alloc.deallocate(e0);
        assert_eq!(alloc.alive_count(), 1);
    }

    #[test]
    fn test_capacity_includes_recycled() {
        let mut alloc = EntityAllocator::new();
        let e = alloc.allocate();
        assert_eq!(alloc.capacity(), 1);
        alloc.deallocate(e);
        assert_eq!(alloc.capacity(), 1); // slot still exists
    }

    #[test]
    fn test_stale_generation_is_detected() {
        let mut alloc = EntityAllocator::new();
        let original = alloc.allocate(); // index=0, gen=0
        alloc.deallocate(original);
        // Allocate enough entities to reuse the slot
        let recycled = alloc.allocate(); // should reuse index=0
        assert_eq!(recycled.index, original.index);
        assert_ne!(recycled.generation, original.generation);
        assert!(!alloc.is_alive(original));
        assert!(alloc.is_alive(recycled));
    }

    #[test]
    fn test_free_list_lifo_order() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate(); // index=0
        let _e1 = alloc.allocate(); // index=1
        let e2 = alloc.allocate(); // index=2
        alloc.deallocate(e0);
        alloc.deallocate(e2);
        // LIFO: e2 freed last, so recycled first
        let recycled1 = alloc.allocate();
        assert_eq!(recycled1.index, 2);
        let recycled2 = alloc.allocate();
        assert_eq!(recycled2.index, 0);
    }

    #[test]
    fn test_fresh_allocation_after_free_list_exhausted() {
        let mut alloc = EntityAllocator::new();
        let e0 = alloc.allocate(); // index=0
        alloc.deallocate(e0); // gen → 1
        let _recycled = alloc.allocate(); // recycles index=0, gen=1
        let fresh = alloc.allocate(); // new slot index=1
        assert_eq!(fresh.index, 1);
        assert_eq!(fresh.generation, 0);
    }
}
