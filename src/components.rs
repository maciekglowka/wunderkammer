use crate::entity::{Entity, IdSize};

#[derive(Default)]
pub struct ComponentStorage<T> {
    dense: Vec<Entity>,
    sparse: Vec<IdSize>,
    components: Vec<T>,
}
impl<T> ComponentStorage<T> {
    pub fn insert(&mut self, entity: Entity, val: T) {}
}
