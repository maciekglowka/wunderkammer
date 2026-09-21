#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

mod components;
mod entity;
mod query;

pub use components::ComponentStorage;
pub use entity::Entity;
pub use query::{Query, QueryMut};

use entity::EntityStorage;
use query::{Fetch, FetchMut};

#[derive(Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Storage<CM> {
    entities: EntityStorage,
    components: CM,
}
impl<CM: ComponentSet + Default> Storage<CM> {
    pub fn new() -> Self {
        Self {
            entities: EntityStorage::default(),
            components: CM::default(),
        }
    }
    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }
    pub fn despawn(&mut self, entity: Entity) {
        self.components.drop_all_components(&entity);
        self.entities.despawn(entity);
    }
    pub fn is_valid(&self, entity: Entity) -> bool {
        self.entities.is_valid(&entity)
    }
    pub fn insert<T>(&mut self, entity: Entity, value: T)
    where
        CM: ComponentHandler<T>,
    {
        self.components.storage_mut().insert(&entity, value);
        self.entities
            .set_component_flag(&entity, <CM as ComponentHandler<T>>::MASK);
    }
    pub fn remove<T>(&mut self, entity: Entity) -> Option<T>
    where
        CM: ComponentHandler<T>,
    {
        if !self.entities.is_valid(&entity) {
            return None;
        }
        self.entities
            .clear_component_flag(&entity, <CM as ComponentHandler<T>>::MASK);
        self.components.storage_mut().remove(&entity)
    }
    pub fn get<'a, T>(&'a self, entity: &Entity) -> Option<T>
    where
        T: Fetch<'a, CM>,
    {
        let (state, _) = T::prepare(&self.components, &self.entities);
        T::get(&state, entity)
    }
    pub fn get_mut<'a, T>(&'a mut self, entity: &Entity) -> Option<T>
    where
        T: FetchMut<'a, CM>,
    {
        let (mut state, _) = unsafe { T::prepare(&raw mut self.components, &self.entities) };
        unsafe { T::get_mut(&mut state, entity) }
    }
    pub fn query<'a, T>(&'a self) -> Query<'a, T, CM>
    where
        T: Fetch<'a, CM>,
    {
        let (state, entities) = T::prepare(&self.components, &self.entities);
        Query::new(state, entities)
    }
    pub fn query_mut<'a, T>(&'a mut self) -> QueryMut<'a, T, CM>
    where
        T: FetchMut<'a, CM>,
    {
        let components = &raw mut self.components;
        let (state, entities) = unsafe { T::prepare(components, &self.entities) };
        QueryMut::new(state, entities)
    }
}

pub unsafe trait ComponentHandler<T> {
    const MASK: u128;

    fn storage(&self) -> &ComponentStorage<T>;
    fn storage_mut(&mut self) -> &mut ComponentStorage<T>;
    unsafe fn storage_raw(c: *mut Self) -> *mut ComponentStorage<T>;
}

pub trait ComponentSet {
    fn drop_all_components(&mut self, entity: &Entity);
}
