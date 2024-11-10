use crate::components::ComponentSet;
use crate::entity::{Entity, EntityStorage};

/// Main storage struct responsible for tracking entities, components and resources.
#[derive(Default)]
pub struct WorldStorage<C, R> {
    entities: EntityStorage,
    pub components: C,
    pub resources: R,
}
impl<C: ComponentSet, R: Default> WorldStorage<C, R> {
    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }
    pub fn despawn(&mut self, entity: Entity) {
        self.components.despawn(entity);
        self.entities.despawn(entity);
    }
}
