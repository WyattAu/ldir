//! Top-level ECS world container.
//!
//! [`World`] holds all entities and component stores. It provides
//! type-erased component registration and access via `TypeId`,
//! along with deterministic query iteration (THM-ECS-DETERMINISM).
//!
//! # References
//!
//! - DEF-WORLD: W = (entities, archetypes, S, allocator)
//! - THM-ECS-DETERMINISM: Entity iteration order is deterministic
//! - AX-ECS-005: Deterministic iteration order

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::ecs::component::ComponentStore;
use crate::ecs::entity::{Entity, EntityAllocator};

/// Top-level ECS container holding all entities and component stores.
///
/// Components are stored in type-erased [`ComponentStore<T>`] instances,
/// keyed by [`TypeId`]. Each component type must be registered with
/// [`register`](World::register) before use (PRE-ECS-001).
///
/// Query iteration yields entities in ascending entity ID order
/// per THM-ECS-DETERMINISM (AX-ECS-005).
///
/// # References
///
/// - DEF-WORLD: W = (entities, archetypes, S, allocator)
/// - THM-ECS-DETERMINISM: Entity iteration order is deterministic
/// - REQ-4.1.4: No Box/Rc/Arc for document nodes (type erasure uses
///   Box internally for heterogeneous storage, but document node
///   components are stored inline in Vecs)
///
/// # Examples
///
    /// ```ignore
    /// use ldir_core::ecs::storage::World;
///
/// #[derive(Debug, PartialEq)]
/// struct Health(i32);
///
/// let mut world = World::new();
/// world.register::<Health>();
///
/// let entity = world.allocate_entity();
/// world.insert_component(entity, Health(100));
///
/// assert_eq!(world.get_component::<Health>(entity), Some(&Health(100)));
/// ```
pub struct World {
    /// Entity allocator with generation tracking.
    entities: EntityAllocator,
    /// Type-erased component stores, keyed by TypeId.
    stores: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    /// Creates a new empty world.
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            stores: HashMap::new(),
        }
    }

    /// Allocates a new entity, returning its versioned ID.
    ///
    /// Per ALG-ECS-CREATE lines 3-8.
    pub fn allocate_entity(&mut self) -> Entity {
        self.entities.allocate()
    }

    /// Returns `true` if the entity reference is still valid.
    ///
    /// Per PRE-ECS-002: checks generation validity.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    /// Deallocates an entity, invalidating its component data.
    ///
    /// Note: This does not remove the entity's components from stores.
    /// Components for dead entities are skipped during iteration.
    /// Per ALG-ECS-DESTROY.
    pub fn deallocate_entity(&mut self, entity: Entity) {
        self.entities.deallocate(entity);
    }

    /// Returns the number of currently live entities.
    pub fn alive_count(&self) -> usize {
        self.entities.alive_count()
    }

    /// Registers a component type, creating its storage.
    ///
    /// Must be called before inserting or querying components of
    /// this type (PRE-ECS-001). Calling register for an already-
    /// registered type is a no-op.
    pub fn register<T: 'static>(&mut self) {
        self.stores
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(ComponentStore::<T>::new()));
    }

    /// Inserts a component for the given entity.
    ///
    /// Panics if the component type has not been registered (PRE-ECS-001).
    pub fn insert_component<T: 'static>(&mut self, entity: Entity, component: T) {
        self.store_mut::<T>()
            .expect("component type not registered; call register::<T>() first")
            .insert(entity, component);
    }

    /// Returns a reference to the component for the given entity.
    ///
    /// Returns `None` if the entity does not have this component or
    /// the component type is not registered.
    ///
    /// Per THM-ECS-ACCESS: O(1) via double indirection.
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.store::<T>()?.get(entity)
    }

    /// Returns a mutable reference to the component for the given entity.
    ///
    /// Per THM-ECS-ACCESS: O(1) via double indirection.
    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.store_mut::<T>()?.get_mut(entity)
    }

    /// Removes the component for the given entity.
    ///
    /// Returns the removed component, or `None` if the entity does
    /// not have this component or the type is not registered.
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Option<T> {
        self.store_mut::<T>()?.remove(entity)
    }

    /// Queries all entities with component `T`, sorted by entity ID.
    ///
    /// Per THM-ECS-DETERMINISM: iteration order is deterministic
    /// (ascending entity ID) regardless of insertion order.
    ///
    /// Per THM-ECS-CACHE-FRIENDLY: touches component data in a
    /// cache-linear pattern after sorting.
    pub fn query<T: 'static>(&self) -> Vec<(Entity, &T)> {
        let store = match self.store::<T>() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let mut items: Vec<(Entity, &T)> = store
            .iter()
            .filter(|(entity, _)| self.entities.is_alive(*entity))
            .collect();
        items.sort_by_key(|(entity, _)| *entity);
        items
    }

    /// Returns the number of entities with component `T`.
    pub fn component_count<T: 'static>(&self) -> usize {
        self.store::<T>().map_or(0, |s| s.len())
    }

    /// Returns `true` if the component type is registered.
    pub fn is_registered<T: 'static>(&self) -> bool {
        self.stores.contains_key(&TypeId::of::<T>())
    }

    fn store<T: 'static>(&self) -> Option<&ComponentStore<T>> {
        self.stores
            .get(&TypeId::of::<T>())?
            .downcast_ref::<ComponentStore<T>>()
    }

    fn store_mut<T: 'static>(&mut self) -> Option<&mut ComponentStore<T>> {
        self.stores
            .get_mut(&TypeId::of::<T>())?
            .downcast_mut::<ComponentStore<T>>()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Health(i32);

    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f64,
        y: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Name(String);

    #[test]
    fn test_register_and_insert_component() {
        let mut world = World::new();
        world.register::<Health>();
        let e = world.allocate_entity();
        world.insert_component(e, Health(100));
        assert_eq!(world.get_component::<Health>(e), Some(&Health(100)));
    }

    #[test]
    fn test_get_nonexistent_component_returns_none() {
        let mut world = World::new();
        world.register::<Health>();
        let e = world.allocate_entity();
        assert_eq!(world.get_component::<Health>(e), None);
    }

    #[test]
    fn test_get_unregistered_component_returns_none() {
        let world = World::new();
        let e = Entity::new(0, 0);
        assert_eq!(world.get_component::<Health>(e), None);
    }

    #[test]
    fn test_insert_multiple_components_on_same_entity() {
        let mut world = World::new();
        world.register::<Health>();
        world.register::<Position>();
        let e = world.allocate_entity();
        world.insert_component(e, Health(100));
        world.insert_component(e, Position { x: 1.0, y: 2.0 });
        assert_eq!(world.get_component::<Health>(e), Some(&Health(100)));
        assert_eq!(
            world.get_component::<Position>(e),
            Some(&Position { x: 1.0, y: 2.0 })
        );
    }

    #[test]
    fn test_update_component() {
        let mut world = World::new();
        world.register::<Health>();
        let e = world.allocate_entity();
        world.insert_component(e, Health(100));
        world.insert_component(e, Health(50));
        assert_eq!(world.get_component::<Health>(e), Some(&Health(50)));
    }

    #[test]
    fn test_get_component_mut() {
        let mut world = World::new();
        world.register::<Health>();
        let e = world.allocate_entity();
        world.insert_component(e, Health(100));
        *world.get_component_mut::<Health>(e).unwrap() = Health(75);
        assert_eq!(world.get_component::<Health>(e), Some(&Health(75)));
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        world.register::<Health>();
        let e = world.allocate_entity();
        world.insert_component(e, Health(100));
        let removed = world.remove_component::<Health>(e).unwrap();
        assert_eq!(removed, Health(100));
        assert_eq!(world.get_component::<Health>(e), None);
    }

    #[test]
    fn test_remove_nonexistent_component_returns_none() {
        let mut world = World::new();
        world.register::<Health>();
        let e = world.allocate_entity();
        assert_eq!(world.remove_component::<Health>(e), None);
    }

    #[test]
    fn test_query_deterministic_order() {
        let mut world = World::new();
        world.register::<Health>();

        // Allocate entities in non-sorted order
        let e3 = world.allocate_entity(); // index=0
        let e1 = world.allocate_entity(); // index=1
        let e2 = world.allocate_entity(); // index=2

        world.insert_component(e3, Health(30));
        world.insert_component(e1, Health(10));
        world.insert_component(e2, Health(20));

        let results = world.query::<Health>();
        // Should be sorted by entity index (0, 1, 2)
        assert_eq!(
            results,
            vec![
                (e3, &Health(30)), // index=0
                (e1, &Health(10)), // index=1
                (e2, &Health(20)), // index=2
            ]
        );
    }

    #[test]
    fn test_query_skips_dead_entities() {
        let mut world = World::new();
        world.register::<Health>();

        let e0 = world.allocate_entity();
        let e1 = world.allocate_entity();
        let e2 = world.allocate_entity();

        world.insert_component(e0, Health(10));
        world.insert_component(e1, Health(20));
        world.insert_component(e2, Health(30));

        world.deallocate_entity(e1);

        let results = world.query::<Health>();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (e0, &Health(10)));
        assert_eq!(results[1], (e2, &Health(30)));
    }

    #[test]
    fn test_query_empty_store() {
        let mut world = World::new();
        world.register::<Health>();
        assert!(world.query::<Health>().is_empty());
    }

    #[test]
    fn test_query_unregistered_returns_empty() {
        let world = World::new();
        assert!(world.query::<Health>().is_empty());
    }

    #[test]
    fn test_query_multiple_component_types_independent() {
        let mut world = World::new();
        world.register::<Health>();
        world.register::<Name>();

        let e0 = world.allocate_entity();
        let e1 = world.allocate_entity();

        world.insert_component(e0, Health(100));
        world.insert_component(e0, Name(String::from("player")));
        world.insert_component(e1, Health(50));

        let health_results = world.query::<Health>();
        let name_results = world.query::<Name>();

        assert_eq!(health_results.len(), 2);
        assert_eq!(name_results.len(), 1);
        assert_eq!(name_results[0], (e0, &Name(String::from("player"))));
    }

    #[test]
    fn test_alive_count() {
        let mut world = World::new();
        assert_eq!(world.alive_count(), 0);
        let e0 = world.allocate_entity();
        let e1 = world.allocate_entity();
        assert_eq!(world.alive_count(), 2);
        world.deallocate_entity(e0);
        assert_eq!(world.alive_count(), 1);
        assert!(world.is_alive(e1));
        assert!(!world.is_alive(e0));
    }

    #[test]
    fn test_register_is_idempotent() {
        let mut world = World::new();
        world.register::<Health>();
        world.register::<Health>(); // should not panic or duplicate
        assert!(world.is_registered::<Health>());
    }

    #[test]
    fn test_is_registered() {
        let mut world = World::new();
        assert!(!world.is_registered::<Health>());
        world.register::<Health>();
        assert!(world.is_registered::<Health>());
    }

    #[test]
    fn test_component_count() {
        let mut world = World::new();
        world.register::<Health>();
        assert_eq!(world.component_count::<Health>(), 0);
        let e = world.allocate_entity();
        world.insert_component(e, Health(100));
        assert_eq!(world.component_count::<Health>(), 1);
    }

    #[test]
    fn test_large_number_of_entities() {
        let mut world = World::new();
        world.register::<Health>();
        let entities: Vec<Entity> = (0..1000).map(|_| world.allocate_entity()).collect();
        for (i, &e) in entities.iter().enumerate() {
            world.insert_component(e, Health(i as i32));
        }
        assert_eq!(world.component_count::<Health>(), 1000);
        let results = world.query::<Health>();
        assert_eq!(results.len(), 1000);
        // Verify deterministic order
        for (i, (entity, health)) in results.iter().enumerate() {
            assert_eq!(entity.index, i as u32);
            assert_eq!(health.0, i as i32);
        }
    }

    #[test]
    fn test_deallocate_and_reallocate() {
        let mut world = World::new();
        world.register::<Health>();
        let e = world.allocate_entity();
        world.insert_component(e, Health(100));
        world.deallocate_entity(e);
        let e2 = world.allocate_entity(); // may reuse slot
        world.insert_component(e2, Health(200));
        assert!(!world.is_alive(e));
        assert!(world.is_alive(e2));
        let results = world.query::<Health>();
        // Dead entity e should be filtered out
        assert!(results.iter().all(|(entity, _)| *entity != e));
    }
}
