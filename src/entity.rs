pub type IdSize = u16;

/// Unique world object identifier.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Entity {
    pub id: IdSize,
    pub version: IdSize,
}

/// EntityStorage responsible for spawning and despawning of the entities.
/// Entity id's are recycled internally and versioned to avoid dead entitiy usage.
/// ```rust
/// use wunderkammer::prelude::*;
/// let mut storage = EntityStorage::default();
/// let a = storage.spawn();
/// let b = storage.spawn();
///
/// storage.despawn(a);
/// let c = storage.spawn();
/// assert_eq!(c.id, a.id);
/// assert_eq!(c.version, a.version + 1);
/// assert_eq!(storage.is_alive(c), true);
/// assert_eq!(storage.is_alive(a), false);
/// ```
#[derive(Default)]
pub struct EntityStorage {
    entities: Vec<Entity>,
    last_recycled: Option<IdSize>,
    first_recycled: Option<IdSize>,
}
impl EntityStorage {
    pub fn spawn(&mut self) -> Entity {
        if let Some(entity) = self.recycle() {
            return entity;
        }
        self.spawn_new()
    }
    pub fn despawn(&mut self, entity: Entity) {
        if self.entities[entity.id as usize].version != entity.version {
            // already despawned!
            return;
        }
        self.entities[entity.id as usize].version += 1;
        if let Some(last) = self.last_recycled {
            // push on the existing recycle list
            self.entities[last as usize].id = entity.id;
        } else {
            // this is the first entity on the recycle list
            self.first_recycled = Some(entity.id);
        }
        // now this one is the prev_recycled
        self.last_recycled = Some(entity.id);
    }
    /// Checks wheter a given entity is still a valid one
    pub fn is_alive(&self, entity: Entity) -> bool {
        let stored = self.entities[entity.id as usize];
        // check if recycled (the id does not match with the index)
        if stored.id != entity.id {
            return false;
        }
        // check if versions match
        stored.version == entity.version
    }
    /// Spawns a fresh entity, with version 0
    fn spawn_new(&mut self) -> Entity {
        let id = self.entities.len();
        let entity = Entity {
            id: id as IdSize,
            version: 0,
        };
        self.entities.push(entity);
        entity
    }
    /// Recycles the previously despawned entity
    fn recycle(&mut self) -> Option<Entity> {
        let recycled_id = self.first_recycled?;
        let recycled = &mut self.entities[recycled_id as usize];

        if self.last_recycled == Some(recycled_id) {
            // no more recycled entities
            self.last_recycled = None;
            self.first_recycled = None;
        } else {
            // the next recycled index was temporarily stored in the id
            self.first_recycled = Some(recycled.id);
        }
        // restore the id to the valid index
        recycled.id = recycled_id;
        Some(*recycled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_new() {
        let mut storage = EntityStorage::default();
        for i in 0..5 {
            let e = storage.spawn_new();
            assert_eq!(i, e.id);
            assert_eq!(0, e.version);
        }

        assert_eq!(storage.entities.len(), 5);
    }

    #[test]
    fn despawn() {
        let mut storage = EntityStorage::default();
        let entities = (0..5).map(|_| storage.spawn_new()).collect::<Vec<_>>();
        storage.despawn(entities[2]);
        assert_eq!(storage.is_alive(entities[2]), false);
    }

    #[test]
    fn recycle_single() {
        let mut storage = EntityStorage::default();
        let a = storage.spawn();
        let _ = storage.spawn();
        storage.despawn(a);
        let c = storage.spawn();
        assert_eq!(a.id, c.id);
        assert_eq!(a.version + 1, c.version);

        storage.despawn(c);
        let d = storage.spawn();
        assert_eq!(a.id, d.id);
        assert_eq!(a.version + 2, d.version);
    }

    #[test]
    fn recycle_many() {
        let mut storage = EntityStorage::default();
        let entities = (0..10).map(|_| storage.spawn_new()).collect::<Vec<_>>();
        storage.despawn(entities[2]);
        storage.despawn(entities[3]);
        storage.despawn(entities[7]);

        let a = storage.spawn();
        assert_eq!(a.id, entities[2].id);
        assert_eq!(a.version, entities[2].version + 1);

        let b = storage.spawn();
        assert_eq!(b.id, entities[3].id);
        assert_eq!(b.version, entities[3].version + 1);

        let c = storage.spawn();
        assert_eq!(c.id, entities[7].id);
        assert_eq!(c.version, entities[7].version + 1);

        // no more entities to recycle
        assert_eq!(storage.spawn().id, 10);
    }

    #[test]
    fn spawn() {
        let mut storage = EntityStorage::default();
        let a = storage.spawn();
        let _ = storage.spawn();

        storage.despawn(a);
        let c = storage.spawn();
        assert_eq!(c.id, a.id);

        let d = storage.spawn();
        assert_eq!(d.id, 2);
    }
}
