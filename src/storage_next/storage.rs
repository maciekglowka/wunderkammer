use crate::storage_next::entity::{ComponentFlag, EntityStorage};

use super::components::ComponentStorage;
use super::entity::Entity;

trait Fetch<'w, CM>
where
    Self: Sized,
{
    fn get(components: &'w CM, entity: &Entity) -> Option<Self>;
    fn entities(_components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
        None
    }
}

impl<'w, A, CM> Fetch<'w, CM> for &'w A
where
    CM: ComponentHandler<A>,
{
    fn get(components: &'w CM, entity: &Entity) -> Option<Self> {
        components.storage().get(entity)
    }
    fn entities(components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
        Some(components.storage().entities())
    }
}
impl<'w, A, CM> Fetch<'w, CM> for Option<&'w A>
where
    CM: ComponentHandler<A>,
{
    fn get(components: &'w CM, entity: &Entity) -> Option<Self> {
        Some(components.storage().get(entity))
    }
}
impl<'w, CM> Fetch<'w, CM> for Entity {
    fn get(_components: &'w CM, entity: &Entity) -> Option<Self> {
        Some(*entity)
    }
}

impl<'w, A, B, CM> Fetch<'w, CM> for (A, B)
where
    A: Fetch<'w, CM>,
    B: Fetch<'w, CM>,
{
    fn get(components: &'w CM, entity: &Entity) -> Option<Self> {
        Some((A::get(components, entity)?, B::get(components, entity)?))
    }
    fn entities(components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
        A::entities(components).or_else(|| B::entities(components))
    }
}

// trait QueryEntities<'w, CM> {
//     fn entities(components: &'w CM) -> Option<std::slice::Iter<'w, Entity>>;
// }

// impl<'w, A, CM> QueryEntities<'w, CM> for &'w A
// where
//     CM: ComponentHandler<A>,
// {
//     fn entities(components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
//         Some(components.storage().entities())
//     }
// }

// impl<'w, A, B, CM> QueryEntities<'w, CM> for (A, B)
// where
//     A: QueryEntities<'w, CM>,
// {
//     fn entities(components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
//         A::entities(components)
//     }
// }

unsafe trait FetchMut<'w, CM>
where
    Self: Sized,
{
    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self>;
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for &'w A
where
    CM: ComponentHandler<A>,
{
    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        components.storage().get(entity)
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for Option<&'w A>
where
    CM: ComponentHandler<A>,
{
    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        Some(components.storage().get(entity))
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for &'w mut A
where
    CM: ComponentHandler<A>,
{
    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        components.storage_mut().get_mut(entity)
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for Option<&'w mut A>
where
    CM: ComponentHandler<A>,
{
    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        Some(components.storage_mut().get_mut(entity))
    }
}
unsafe impl<'w, CM> FetchMut<'w, CM> for Entity {
    unsafe fn get_mut(_components: &'w mut CM, entity: &Entity) -> Option<Self> {
        Some(*entity)
    }
}

unsafe impl<'w, A, B, CM> FetchMut<'w, CM> for (A, B)
where
    A: FetchMut<'w, CM>,
    B: FetchMut<'w, CM>,
{
    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        // FIXME - component collision check
        let components = &raw mut *components;
        Some((
            A::get_mut(components.as_mut_unchecked(), entity)?,
            B::get_mut(components.as_mut_unchecked(), entity)?,
        ))
    }
}

pub struct Query<'w, F, CM>
where
    F: Fetch<'w, CM>,
{
    entities: EntityIter<'w>,
    components: &'w CM,
    _marker: std::marker::PhantomData<F>,
}
impl<'w, F, CM> Iterator for Query<'w, F, CM>
where
    F: Fetch<'w, CM>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity, flags)) = self.entities.next() {
            if let Some(f) = F::get(&self.components, entity) {
                return Some(f);
            }
        }
        None
    }
}

pub struct QueryMut<'w, F> {
    entities: EntityIter<'w>,
    fetch: F,
}
// impl<'w, F> Iterator for QueryMut<'w, F>
// where
//     F: FetchMut<'w>,
// {
//     type Item = F::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some((entity, flags)) = self.entities.next() {
//             if let Some(f) = unsafe { self.fetch.get_mut(entity) } {
//                 return Some(f);
//             }
//         }
//         None
//     }
// }

/// Iterate over entities together with component flags.
struct EntityIter<'w> {
    inner: Option<std::slice::Iter<'w, Entity>>,
    // TODO
    flags: &'w [ComponentFlag],
}
impl<'w> Iterator for EntityIter<'w> {
    type Item = (&'w Entity, ComponentFlag);

    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.inner.as_mut()?.next()?;
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
        self.components.storage_mut().__insert(entity, value);
    }
    pub fn get<'a, T>(&'a self, entity: &Entity) -> Option<T>
    where
        T: Fetch<'a, CM>,
    {
        T::get(&self.components, entity)
    }
    pub fn get_mut<'a, T>(&'a mut self, entity: &Entity) -> Option<T>
    where
        T: FetchMut<'a, CM>,
    {
        unsafe { T::get_mut(&mut self.components, entity) }
    }
    pub fn query<'a, T>(&'a self) -> Query<'a, T, CM>
    where
        T: Fetch<'a, CM>,
    {
        Query {
            entities: EntityIter {
                inner: T::entities(&self.components),
                flags: &self.entities.component_flags,
            },
            components: &self.components,
            _marker: std::marker::PhantomData::default(),
        }
    }
    // pub fn query_mut<'a, T>(&'a mut self) -> QueryMut<'a, T::FetchMut>
    // where
    //     T: IntoFetch<'a, CM>,
    // {
    //     T::query_mut(self)
    // }
}

pub trait ComponentHandler<T> {
    const MASK: u128;

    fn storage(&self) -> &ComponentStorage<T>;
    fn storage_mut(&mut self) -> &mut ComponentStorage<T>;
    fn storage_raw(&mut self) -> *mut ComponentStorage<T>;
}
