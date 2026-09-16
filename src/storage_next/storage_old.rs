use crate::storage_next::entity::{ComponentFlag, EntityStorage};

use super::components::ComponentStorage;
use super::entity::Entity;

trait QuerySource {
    type Item;

    fn get(&self, entity: &Entity) -> Option<Self::Item>;
}
unsafe trait QuerySourceMut<'w> {
    type Item;

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item>;
}
impl<'w, A> QuerySource for &'w ComponentStorage<A> {
    type Item = &'w A;

    fn get(&self, entity: &Entity) -> Option<Self::Item> {
        ComponentStorage::get(self, entity)
    }
}
unsafe impl<'w, A: 'static> QuerySourceMut<'w> for &'w ComponentStorage<A> {
    type Item = &'w A;

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item> {
        ComponentStorage::get(self, entity)
    }
}
unsafe impl<'w, A: 'static> QuerySourceMut<'w> for *mut ComponentStorage<A> {
    type Item = &'w mut A;

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item> {
        ComponentStorage::get_mut(self.as_mut_unchecked(), entity)
    }
}

pub trait Fetch {
    type Item;

    fn get(&self, entity: &Entity) -> Option<Self::Item>;
}
pub unsafe trait FetchMut<'w> {
    type Item;

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item>;
}

impl<T> Fetch for T
where
    T: QuerySource,
{
    type Item = T::Item;

    fn get(&self, entity: &Entity) -> Option<Self::Item> {
        self.get(entity)
    }
}
unsafe impl<'w, T> FetchMut<'w> for T
where
    T: QuerySourceMut<'w>,
{
    type Item = T::Item;

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item> {
        self.get_mut(entity)
    }
}

impl<'w, A: QuerySource, B: QuerySource> Fetch for (A, B) {
    type Item = (A::Item, B::Item);

    fn get(&self, entity: &Entity) -> Option<Self::Item> {
        Some((self.0.get(entity)?, self.1.get(entity)?))
    }
}
unsafe impl<'w, A: QuerySourceMut<'w>, B: QuerySourceMut<'w>> FetchMut<'w> for (A, B) {
    type Item = (A::Item, B::Item);

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item> {
        Some((self.0.get_mut(entity)?, self.1.get_mut(entity)?))
    }
}

// pub struct Query<'w, F> {
//     entities: EntityIter<'w>,
//     fetch: F,
// }
// impl<'w, F> Iterator for Query<'w, F>
// where
//     F: Fetch<'w>,
// {
//     type Item = F::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some((entity, flags)) = self.entities.next() {
//             if let Some(f) = self.fetch.get(entity) {
//                 return Some(f);
//             }
//         }
//         None
//     }
// }

// pub struct QueryMut<'w, F> {
//     entities: EntityIter<'w>,
//     fetch: F,
// }
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
    inner: std::slice::Iter<'w, Entity>,
    // TODO
    flags: &'w [ComponentFlag],
}
impl<'w> Iterator for EntityIter<'w> {
    type Item = (&'w Entity, ComponentFlag);

    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.inner.next()?;
        let flag = self.flags[entity.id as usize];
        Some((entity, flag))
    }
}

pub trait IntoFetch<'w, CM> {
    type Fetch: Fetch;

    fn into_fetch(components: &'w CM) -> Self::Fetch;

    // fn query(storage: &'w Storage<CM>) -> Query<'w, Self::Fetch> {
    //     let fetch = Self::into_fetch(&storage.components);
    //     let entities = fetch.entities();

    //     Query {
    //         entities: EntityIter {
    //             inner: entities,
    //             flags: &storage.entities.component_flags,
    //         },
    //         fetch,
    //     }
    // }
}

// pub trait IntoFetchMut<'w, CM> {
//     type FetchMut: FetchMut<'w>;

//     fn into_fetch_mut(components: &'w mut CM) -> Self::FetchMut;
//     fn query_mut(storage: &'a mut Storage<CM>) -> QueryMut<'a,
// Self::FetchMut> {         let fetch = Self::into_fetch_mut(&mut
// storage.components);         let entities = fetch.entities();

//         QueryMut {
//             entities: EntityIter {
//                 inner: entities,
//                 flags: &storage.entities.component_flags,
//             },
//             fetch,
//         }
//     }
// }

impl<'w, A, CM> IntoFetch<'w, CM> for &A
where
    A: 'static,
    CM: ComponentHandler<A>,
    // CM: QuerySource,
{
    type Fetch = &'w ComponentStorage<A>;

    fn into_fetch(components: &'w CM) -> Self::Fetch {
        components.storage()
    }
}

// impl<'a, A, CM> IntoFetchMut<'a, CM> for &mut A
// where
//     A: 'static,
//     CM: ComponentHandler<A>,
// {
//     type FetchMut = *mut ComponentStorage<A>;

//     fn into_fetch_mut(components: &'a mut CM) -> Self::FetchMut {
//         components.storage_raw()
//     }
// }

impl<'w, A, B, CM> IntoFetch<'w, CM> for (&A, &B)
where
    A: QuerySource + IntoFetch<'w, CM>,
    B: QuerySource + IntoFetch<'w, CM>,
    CM: ComponentHandler<A>,
    CM: ComponentHandler<B>,
{
    type Fetch = (A::Fetch, B::Fetch);

    fn into_fetch(components: &'w CM) -> Self::Fetch {
        // (
        //     <CM as ComponentHandler<A>>::storage(&storage.components),
        //     <CM as ComponentHandler<B>>::storage(&storage.components),
        // )
        (A::into_fetch(components), B::into_fetch(components))
    }
    // fn query(storage: &'a Storage<CM>) -> Query<'a, Self::Fetch> {
    //     let entities = <CM as
    // ComponentHandler<A>>::storage(&storage.components).entities();
    //     Query {
    //         entities: EntityIter {
    //             inner: entities,
    //             flags: &storage.entities.component_flags,
    //         },
    //         fetch: Self::into_fetch(storage),
    //     }
    // }
}

// impl<'a, A, B, CM> IntoFetchMut<'a, CM> for (A, B)
// where
//     A: IntoFetchMut<'a, CM> + 'static,
//     B: IntoFetchMut<'a, CM>,
//     CM: ComponentHandler<A>,
// {
//     type FetchMut = (A::FetchMut, B::FetchMut);

//     fn into_fetch_mut(storage: &'a mut CM) -> Self::FetchMut {
//         // TODO
//         // const {
//         //     if <CM as ComponentHandler<A>>::MASK & <CM as
//         // ComponentHandler<B>>::MASK != 0 {         panic!("Duplicated
//         // query type");     }
//         // }

//         // (
//         //     <CM as ComponentHandler<A>>::storage_raw(&mut
//         // storage.components),     <CM as
//         // ComponentHandler<B>>::storage_raw(&mut storage.components), )
//         (A::into_fetch_mut(storage), B::into_fetch_mut(storage))
//     }
//     fn query_mut(storage: &'a mut Storage<CM>) -> QueryMut<'a,
// Self::FetchMut> {         let fetch = Self::into_fetch_mut(storage);
//         let entities = <CM as
// ComponentHandler<A>>::storage(&storage.components).entities();
//         QueryMut {
//             entities: EntityIter {
//                 inner: entities,
//                 flags: &storage.entities.component_flags,
//             },
//             fetch,
//         }
//     }
// }

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
