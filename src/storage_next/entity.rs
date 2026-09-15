// TODO rewrite once settles.

pub type IdSize = u16;
pub type ComponentFlag = u16;

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Entity {
    pub id: IdSize,
    pub version: IdSize,
}

#[derive(Default)]
pub(crate) struct EntityStorage {
    entities: Vec<Entity>,
    pub(crate) component_flags: Vec<ComponentFlag>,
    last_recycled: Option<IdSize>,
    first_recycled: Option<IdSize>,
}
impl EntityStorage {
    /// Spawn an Entity
    pub(crate) fn spawn(&mut self) -> Entity {
        if let Some(entity) = self.recycle() {
            return entity;
        }
        self.spawn_new()
    }
    /// Despawn Entity from the storage
    pub(crate) fn despawn(&mut self, entity: Entity) {
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
    /// Validates the given entity
    pub(crate) fn is_valid(&self, entity: &Entity) -> bool {
        let Some(existing) = self.entities.get(entity.id as usize) else {
            return false;
        };
        existing.version == entity.version
            && existing.id == entity.id
            && self.last_recycled != Some(entity.id)
            && self.first_recycled != Some(entity.id)
    }
    /// Iterate through valid entities
    pub(crate) fn all(&self) -> impl Iterator<Item = &Entity> + use<'_> {
        self.entities.iter().filter(|e| self.is_valid(e))
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
