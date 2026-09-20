use super::components::ComponentStorage;
use super::entity::{ComponentFlag, Entity, EntityStorage};
use super::query::{Fetch, Query};
use super::query_mut::{FetchMut, QueryMut};

/// Iterate over entities together with component flags.
pub(crate) struct EntityIter<'w> {
    pub(crate) inner: std::slice::Iter<'w, Entity>,
    // TODO
    pub(crate) flags: &'w [ComponentFlag],
}
impl<'w> Iterator for EntityIter<'w> {
    type Item = (&'w Entity, ComponentFlag);

    fn next(&mut self) -> Option<Self::Item> {
        // FIXME for shared queries.
        let entity = self.inner.next()?;
        // let entity = self.inner.as_mut()?.next()?;
        let flag = self.flags[entity.id as usize];
        Some((entity, flag))
    }
}

pub struct Storage<CM> {
    entities: EntityStorage,
    components: CM,
}
impl<CM: Default> Storage<CM> {
    pub fn new() -> Self {
        Self {
            entities: super::entity::EntityStorage::default(),
            components: CM::default(),
        }
    }
    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }
    pub fn insert<T>(&mut self, entity: Entity, value: T)
    where
        CM: ComponentHandler<T>,
    {
        self.components.storage_mut().insert(entity, value);
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
