#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::entity::{Entity, IdSize};
use std::collections::HashSet;

const TOMBSTONE: IdSize = IdSize::MAX;

/// Base trait for the `components` world field.
/// Handles component cleanup after an entity is despawned from the world.
pub trait ComponentSet {
    /// despawn all the entity's components
    fn despawn(&mut self, entity: Entity);
}

/// Component storage based on a sparse set data structure.
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ComponentStorage<T> {
    dense: Vec<Entity>,
    sparse: Vec<IdSize>,
    values: Vec<T>,
}
impl<T> ComponentStorage<T> {
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.values.get(self.get_dense_index(entity)?)
    }
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let i = self.get_dense_index(entity)?;
        self.values.get_mut(i)
    }
    // Return currently stored entities
    pub fn entities(&self) -> HashSet<Entity> {
        HashSet::from_iter(self.dense.iter().copied())
    }
    // Insert a new component for the entity.
    // Overwrite if already exists.
    pub fn insert(&mut self, entity: Entity, value: T) {
        let index = entity.id as usize;
        if index >= self.sparse.len() {
            // fill empty values with tombstones
            self.sparse.resize(index + 1, TOMBSTONE);
        }

        // sparse array points to the element in the dense one
        self.sparse[index] = self.dense.len() as IdSize;
        // we push the element at the end of the dense array
        self.dense.push(entity);
        // components array is kept in sync with the dense array
        self.values.push(value);
    }

    // Removes component for a given entity
    // Keeps the values densly packed
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let removed_idx = self.get_dense_index(entity)?;

        // we are going to swap the removed value with the last one first
        let last_idx = self.dense.len() - 1;
        let swapped_sparse_idx = self.dense[last_idx].id as usize;

        self.dense.swap(removed_idx, last_idx);
        self.values.swap(removed_idx, last_idx);

        // now remove the last element
        let _ = self.dense.pop();
        let removed = self.values.pop();

        // now fix the sparse vec
        self.sparse[swapped_sparse_idx] = removed_idx as IdSize;
        self.sparse[entity.id as usize] = TOMBSTONE;

        removed
    }

    fn get_dense_index(&self, entity: Entity) -> Option<usize> {
        let i = *self.sparse.get(entity.id as usize)? as usize;
        // validate version
        match *self.dense.get(i)? == entity {
            false => None,
            true => Some(i),
        }
    }
}
impl<T> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self {
            dense: Vec::new(),
            sparse: Vec::new(),
            values: Vec::new(),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn insert_first() {
        let mut storage = ComponentStorage::default();
        let entity = Entity { id: 0, version: 0 };
        storage.insert(entity, "VALUE");

        assert_eq!(storage.dense.len(), 1);
        assert_eq!(storage.values.len(), 1);
        assert_eq!(storage.get(entity), Some(&"VALUE"));
    }

    #[test]
    fn insert_many() {
        let mut storage = ComponentStorage::default();
        for i in 0..10 {
            // make non contiguous
            if i % 2 == 0 {
                continue;
            }
            let entity = Entity { id: i, version: 0 };
            storage.insert(entity, 10 * i);
        }

        assert_eq!(storage.dense.len(), 5);
        assert_eq!(storage.values.len(), 5);
        assert_eq!(storage.sparse.len(), 10);
        assert_eq!(storage.entities().len(), 5);

        for i in 0..10 {
            let entity = Entity { id: i, version: 0 };
            if i % 2 == 0 {
                assert_eq!(storage.get(entity), None);
            } else {
                assert_eq!(storage.get(entity), Some(&(10 * i)));
            }
        }
    }

    #[test]
    fn contains() {
        let mut storage = ComponentStorage::default();
        let entity = Entity { id: 3, version: 0 };
        storage.insert(entity, "VALUE");
        assert_eq!(storage.get_dense_index(entity), Some(0));
    }

    #[test]
    fn does_not_contain() {
        let mut storage = ComponentStorage::default();
        let entity = Entity { id: 3, version: 0 };
        storage.insert(entity, "VALUE");
        let other = Entity { id: 1, version: 0 };
        assert_eq!(storage.get_dense_index(other), None);
    }

    #[test]
    fn does_not_contain_exceed_index() {
        let mut storage = ComponentStorage::default();
        let entity = Entity { id: 3, version: 0 };
        storage.insert(entity, "VALUE");
        let other = Entity { id: 10, version: 0 };
        assert_eq!(storage.get_dense_index(other), None);
    }

    #[test]
    fn remove_single() {
        let mut storage = ComponentStorage::default();
        let entity = Entity { id: 0, version: 0 };
        storage.insert(entity, "VALUE");
        storage.remove(entity);

        assert_eq!(storage.dense.len(), 0);
        assert_eq!(storage.values.len(), 0);
        assert_eq!(storage.get(entity), None);
    }

    #[test]
    fn remove_many() {
        let mut storage = ComponentStorage::default();
        for i in 0..10 {
            let entity = Entity { id: i, version: 0 };
            storage.insert(entity, 10 * i);
        }
        assert_eq!(storage.dense.len(), 10);
        assert_eq!(storage.values.len(), 10);
        assert_eq!(storage.entities().len(), 10);

        for i in 0..10 {
            let entity = Entity { id: i, version: 0 };
            if i % 2 == 0 {
                storage.remove(entity);
            }
        }

        assert_eq!(storage.dense.len(), 5);
        assert_eq!(storage.values.len(), 5);
        assert_eq!(storage.entities().len(), 5);

        for i in 0..10 {
            let entity = Entity { id: i, version: 0 };
            if i % 2 == 0 {
                assert_eq!(storage.get(entity), None);
            } else {
                assert_eq!(storage.get(entity), Some(&(10 * i)));
            }
        }
    }

    #[test]
    fn get_wrong_version() {
        let mut storage = ComponentStorage::default();
        let entity = Entity { id: 0, version: 1 };
        storage.insert(entity, "VALUE");

        assert_eq!(storage.get(Entity { id: 0, version: 0 }), None);
    }

    #[test]
    fn get_wrong_id() {
        let mut storage = ComponentStorage::default();
        let entity = Entity { id: 3, version: 1 };
        storage.insert(entity, "VALUE");

        assert_eq!(storage.get(Entity { id: 0, version: 1 }), None);
    }
}
