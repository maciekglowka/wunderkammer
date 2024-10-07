use crate::components::Components;
use crate::entity::{Entity, EntityStorage};

#[derive(Default)]
pub struct WorldStorage<C> {
    entities: EntityStorage,
    pub components: C,
}
impl<C: Components> WorldStorage<C> {
    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }
    pub fn despawn(&mut self, entity: Entity) {
        self.components.despawn(entity);
        self.entities.despawn(entity);
    }
}
