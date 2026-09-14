// TODO rewrite

use super::entity::{Entity, IdSize};
const TOMBSTONE: IdSize = IdSize::MAX;

pub struct ComponentStorage<T> {
    dense: Vec<Entity>,
    sparse: Vec<IdSize>,
    values: Vec<T>,
}
impl<T> ComponentStorage<T> {
    pub fn get(&self, entity: &Entity) -> Option<&T> {
        self.values.get(self.get_dense_index(entity)?)
    }
    pub fn get_mut(&mut self, entity: &Entity) -> Option<&mut T> {
        let i = self.get_dense_index(entity)?;
        self.values.get_mut(i)
    }
    // Return currently stored entities
    pub fn entities(&self) -> std::slice::Iter<Entity> {
        self.dense.iter()
    }
    // Insert a new component for the entity.
    //
    // Overwrite if already exists.
    // Since it cannot validate the entity,
    // it is recommended to use `insert!` macro that calls it internally.
    pub fn __insert(&mut self, entity: Entity, value: T) {
        // check if replacement
        if let Some(index) = self.get_dense_index(&entity) {
            self.values[index] = value;
            return;
        }

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
    // Keeps the values densely packed
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let removed_idx = self.get_dense_index(&entity)?;

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

    fn get_dense_index(&self, entity: &Entity) -> Option<usize> {
        let i = *self.sparse.get(entity.id as usize)? as usize;
        // validate version
        match self.dense.get(i)? == entity {
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
