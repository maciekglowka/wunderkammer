use crate::storage_next::entity::{ComponentFlag, EntityStorage};

use super::components::ComponentStorage;
use super::entity::Entity;

pub trait Fetch {
    type Item;

    fn get(&self, entity: &Entity) -> Option<Self::Item>;
}
pub unsafe trait FetchMut<'w> {
    type Item;

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item>;
}

impl<'w, A> Fetch for &'w ComponentStorage<A> {
    type Item = &'w A;

    fn get(&self, entity: &Entity) -> Option<Self::Item> {
        ComponentStorage::get(self, entity)
    }
}
unsafe impl<'w, A: 'static> FetchMut<'w> for *mut ComponentStorage<A> {
    type Item = &'w mut A;

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item> {
        ComponentStorage::get_mut(self.as_mut_unchecked(), entity)
    }
}

impl<A: Fetch, B: Fetch> Fetch for (A, B) {
    type Item = (A::Item, B::Item);

    fn get(&self, entity: &Entity) -> Option<Self::Item> {
        Some((self.0.get(entity)?, self.1.get(entity)?))
    }
}
unsafe impl<'w, A: FetchMut<'w>, B: FetchMut<'w>> FetchMut<'w> for (A, B) {
    type Item = (A::Item, B::Item);

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item> {
        Some((self.0.get_mut(entity)?, self.1.get_mut(entity)?))
    }
}

pub struct Query<'a, F> {
    entities: EntityIter<'a>,
    fetch: F,
}
impl<'a, F> Iterator for Query<'a, F>
where
    F: Fetch,
{
    type Item = F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity, flags)) = self.entities.next() {
            if let Some(f) = self.fetch.get(entity) {
                return Some(f);
            }
        }
        None
    }
}

pub struct QueryMut<'a, F> {
    entities: EntityIter<'a>,
    fetch: F,
}
impl<'a, F> Iterator for QueryMut<'a, F>
where
    F: FetchMut<'a>,
{
    type Item = F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity, flags)) = self.entities.next() {
            if let Some(f) = unsafe { self.fetch.get_mut(entity) } {
                return Some(f);
            }
        }
        None
    }
}

/// Iterate over entities together with component flags.
struct EntityIter<'a> {
    inner: std::slice::Iter<'a, Entity>,
    // TODO
    flags: &'a [ComponentFlag],
}
impl<'a> Iterator for EntityIter<'a> {
    type Item = (&'a Entity, ComponentFlag);

    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.inner.next()?;
        let flag = self.flags[entity.id as usize];
        Some((entity, flag))
    }
}

pub trait IntoFetch<'a, CM> {
    type Fetch: Fetch;
    type FetchMut: FetchMut<'a>;

    fn into(storage: &'a Storage<CM>) -> Self::Fetch;
    fn into_mut(storage: &'a mut Storage<CM>) -> Self::FetchMut;
    fn query(storage: &'a Storage<CM>) -> Query<'a, Self::Fetch>;
    fn query_mut(storage: &'a mut Storage<CM>) -> QueryMut<'a, Self::FetchMut>;
}

// Using &A to avoid conflicitng impl with tuples - to be tested if works.
impl<'a, A, CM> IntoFetch<'a, CM> for A
where
    A: 'static,
    CM: ComponentHandler<A>,
{
    type Fetch = &'a ComponentStorage<A>;
    type FetchMut = *mut ComponentStorage<A>;

    fn into(storage: &'a Storage<CM>) -> Self::Fetch {
        <CM as ComponentHandler<A>>::storage(&storage.components)
    }

    fn into_mut(storage: &'a mut Storage<CM>) -> Self::FetchMut {
        <CM as ComponentHandler<A>>::storage_raw(&mut storage.components)
    }

    fn query(storage: &'a Storage<CM>) -> Query<'a, Self::Fetch> {
        let entities = <CM as ComponentHandler<A>>::storage(&storage.components).entities();
        Query {
            entities: EntityIter {
                inner: entities,
                flags: &storage.entities.component_flags,
            },
            fetch: <Self as IntoFetch<CM>>::into(storage),
        }
    }
    fn query_mut(storage: &'a mut Storage<CM>) -> QueryMut<'a, Self::FetchMut> {
        let fetch = <Self as IntoFetch<CM>>::into_mut(storage);
        let entities = <CM as ComponentHandler<A>>::storage(&storage.components).entities();
        QueryMut {
            entities: EntityIter {
                inner: entities,
                flags: &storage.entities.component_flags,
            },
            fetch,
        }
    }
}

impl<'a, A, B, CM> IntoFetch<'a, CM> for (A, B)
where
    A: 'static,
    B: 'static,
    CM: ComponentHandler<A>,
    CM: ComponentHandler<B>,
{
    type Fetch = (&'a ComponentStorage<A>, &'a ComponentStorage<B>);
    type FetchMut = (*mut ComponentStorage<A>, *mut ComponentStorage<B>);

    fn into(storage: &'a Storage<CM>) -> Self::Fetch {
        (
            <CM as ComponentHandler<A>>::storage(&storage.components),
            <CM as ComponentHandler<B>>::storage(&storage.components),
        )
    }
    fn into_mut(storage: &'a mut Storage<CM>) -> Self::FetchMut {
        const {
            if <CM as ComponentHandler<A>>::MASK & <CM as ComponentHandler<B>>::MASK != 0 {
                panic!("Duplicated query type");
            }
        }
        (
            <CM as ComponentHandler<A>>::storage_raw(&mut storage.components),
            <CM as ComponentHandler<B>>::storage_raw(&mut storage.components),
        )
    }
    fn query(storage: &'a Storage<CM>) -> Query<'a, Self::Fetch> {
        let entities = <CM as ComponentHandler<A>>::storage(&storage.components).entities();
        Query {
            entities: EntityIter {
                inner: entities,
                flags: &storage.entities.component_flags,
            },
            fetch: <Self as IntoFetch<CM>>::into(storage),
        }
    }
    fn query_mut(storage: &'a mut Storage<CM>) -> QueryMut<'a, Self::FetchMut> {
        let fetch = <Self as IntoFetch<CM>>::into_mut(storage);
        let entities = <CM as ComponentHandler<A>>::storage(&storage.components).entities();
        QueryMut {
            entities: EntityIter {
                inner: entities,
                flags: &storage.entities.component_flags,
            },
            fetch,
        }
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
    pub fn get<'a, T>(&'a self, entity: &Entity) -> Option<<T::Fetch as Fetch>::Item>
    where
        T: IntoFetch<'a, CM>,
    {
        T::into(self).get(entity)
    }
    pub fn get_mut<'a, T>(&'a mut self, entity: &Entity) -> Option<<T::FetchMut as FetchMut>::Item>
    where
        T: IntoFetch<'a, CM>,
    {
        unsafe { T::into_mut(self).get_mut(entity) }
    }
    pub fn query<'a, T>(&'a self) -> Query<'a, T::Fetch>
    where
        T: IntoFetch<'a, CM>,
    {
        T::query(self)
    }
    pub fn query_mut<'a, T>(&'a mut self) -> QueryMut<'a, T::FetchMut>
    where
        T: IntoFetch<'a, CM>,
    {
        T::query_mut(self)
    }
}

pub trait ComponentHandler<T> {
    const MASK: u128;

    fn storage(&self) -> &ComponentStorage<T>;
    fn storage_mut(&mut self) -> &mut ComponentStorage<T>;
    fn storage_raw(&mut self) -> *mut ComponentStorage<T>;
}
