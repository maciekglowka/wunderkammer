use crate::storage_next::components::get_dense_index;
use crate::storage_next::entity::{ComponentFlag, EntityStorage, IdSize};

use super::components::ComponentStorage;
use super::entity::Entity;

// TODO Seal traits

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

unsafe trait FetchMut<'w, CM>
where
    Self: Sized,
{
    type View;
    const COLLISION_MASK: u128;

    unsafe fn prepare(
        components: *mut CM,
        entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter<'w>>);
    unsafe fn get_mut(view: &mut Self::View, entity: &Entity) -> Option<Self>;
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for &'w A
where
    CM: ComponentHandler<A>,
{
    type View = ViewMut<'w, A>;
    const COLLISION_MASK: u128 = <CM as ComponentHandler<A>>::MASK;

    unsafe fn prepare(
        components: *mut CM,
        entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter>) {
        const {
            assert!(
                <Self as FetchMut<'w, CM>>::COLLISION_MASK != 0,
                "component collision mask can't be 0"
            )
        };

        let storage = &mut *ComponentHandler::<A>::storage_raw(components);
        let (sparse, dense, values) = storage.parts();
        let entities = EntityIter {
            inner: dense.iter(),
            flags: &entity_storage.component_flags,
        };
        (
            ViewMut {
                sparse,
                dense,
                values,
                _marker: std::marker::PhantomData,
            },
            Some(entities),
        )
    }

    unsafe fn get_mut(view: &mut Self::View, entity: &Entity) -> Option<Self> {
        view.get_mut(entity).map(|a| &*a)
    }

    // fn next_entity(view: &mut Self::View) -> Option<Option<Entity>> {
    //     let entity = view.dense.get(view.entity_idx).copied();
    //     view.entity_idx += 1;
    //     Some(entity)
    // }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for Option<&'w A>
where
    CM: ComponentHandler<A>,
{
    type View = ViewMut<'w, A>;
    const COLLISION_MASK: u128 = <CM as ComponentHandler<A>>::MASK;

    unsafe fn prepare(
        components: *mut CM,
        _entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter>) {
        const {
            assert!(
                <Self as FetchMut<'w, CM>>::COLLISION_MASK != 0,
                "component collision mask can't be 0"
            )
        };

        let storage = &mut *ComponentHandler::<A>::storage_raw(components);
        let (sparse, dense, values) = storage.parts();
        (
            ViewMut {
                sparse,
                dense,
                values,
                _marker: std::marker::PhantomData,
            },
            None,
        )
    }

    unsafe fn get_mut(view: &mut Self::View, entity: &Entity) -> Option<Self> {
        Some(view.get_mut(entity).map(|a| &*a))
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for &'w mut A
where
    CM: ComponentHandler<A>,
{
    type View = ViewMut<'w, A>;
    const COLLISION_MASK: u128 = <CM as ComponentHandler<A>>::MASK;

    unsafe fn prepare(
        components: *mut CM,
        entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter>) {
        const {
            assert!(
                <Self as FetchMut<'w, CM>>::COLLISION_MASK != 0,
                "component collision mask can't be 0"
            )
        };

        let storage = &mut *ComponentHandler::<A>::storage_raw(components);
        let (sparse, dense, values) = storage.parts();
        let entities = EntityIter {
            inner: dense.iter(),
            flags: &entity_storage.component_flags,
        };
        (
            ViewMut {
                sparse,
                dense,
                values,
                _marker: std::marker::PhantomData,
            },
            Some(entities),
        )
    }

    unsafe fn get_mut(view: &mut Self::View, entity: &Entity) -> Option<Self> {
        view.get_mut(entity)
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for Option<&'w mut A>
where
    CM: ComponentHandler<A>,
{
    type View = ViewMut<'w, A>;
    const COLLISION_MASK: u128 = <CM as ComponentHandler<A>>::MASK;

    unsafe fn prepare(
        components: *mut CM,
        _entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter>) {
        const {
            assert!(
                <Self as FetchMut<'w, CM>>::COLLISION_MASK != 0,
                "component collision mask can't be 0"
            )
        };

        let storage = &mut *ComponentHandler::<A>::storage_raw(components);
        let (sparse, dense, values) = storage.parts();
        (
            ViewMut {
                sparse,
                dense,
                values,
                _marker: std::marker::PhantomData,
            },
            None,
        )
    }
    unsafe fn get_mut(view: &mut Self::View, entity: &Entity) -> Option<Self> {
        Some(view.get_mut(entity))
    }
}
unsafe impl<'w, CM> FetchMut<'w, CM> for Entity {
    type View = ();
    const COLLISION_MASK: u128 = 0;

    unsafe fn prepare(
        _components: *mut CM,
        _entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter>) {
        ((), None)
    }

    unsafe fn get_mut(_view: &mut Self::View, entity: &Entity) -> Option<Self> {
        Some(*entity)
    }
}

unsafe impl<'w, A, B, CM> FetchMut<'w, CM> for (A, B)
where
    A: FetchMut<'w, CM>,
    B: FetchMut<'w, CM>,
{
    type View = (A::View, B::View);
    const COLLISION_MASK: u128 = A::COLLISION_MASK | B::COLLISION_MASK;

    unsafe fn prepare(
        components: *mut CM,
        entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter>) {
        const {
            assert!(
                A::COLLISION_MASK & B::COLLISION_MASK == 0,
                "conflicting query type"
            );
        };
        let (ca, ea) = A::prepare(components, entity_storage);
        let (cb, eb) = B::prepare(components, entity_storage);
        let entities = ea.or_else(|| eb);
        ((ca, cb), entities)
    }

    unsafe fn get_mut(view: &mut Self::View, entity: &Entity) -> Option<Self> {
        let (va, vb) = view;
        Some((A::get_mut(va, entity)?, B::get_mut(vb, entity)?))
    }
}

// struct View<'w, A> {

// }

struct ViewMut<'w, A> {
    /// Used for iteration over component storage entities.
    sparse: &'w [IdSize],
    dense: &'w [Entity],
    values: *mut A,
    _marker: std::marker::PhantomData<&'w mut [A]>,
}
impl<'w, A> ViewMut<'w, A> {
    /// Up to the caller to guarantee that
    /// same entity is not passed twice.
    /// Should be only used in query iterator.
    unsafe fn get_mut(&mut self, entity: &Entity) -> Option<&'w mut A> {
        let idx = get_dense_index(&self.sparse, &self.dense, entity)?;
        Some(&mut *self.values.add(idx))
    }
}

pub struct Query<'w, F, CM>
where
    F: Fetch<'w, CM>,
{
    entities: Option<EntityIter<'w>>,
    components: &'w CM,
    _marker: std::marker::PhantomData<F>,
}
impl<'w, F, CM> Iterator for Query<'w, F, CM>
where
    F: Fetch<'w, CM>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity, flags)) = self.entities.as_mut()?.next() {
            if let Some(f) = F::get(&self.components, entity) {
                return Some(f);
            }
        }
        None
    }
}

pub struct QueryMut<'w, F, CM>
where
    F: FetchMut<'w, CM>,
{
    state: F::View,
    entities: Option<EntityIter<'w>>,
    // TODO validate marker
    _marker: std::marker::PhantomData<fn() -> (F, &'w CM)>,
}
impl<'w, F, CM: 'w> Iterator for QueryMut<'w, F, CM>
where
    F: FetchMut<'w, CM>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity, flags)) = self.entities.as_mut()?.next() {
            if let Some(f) = unsafe { F::get_mut(&mut self.state, entity) } {
                return Some(f);
            }
        }
        None
    }
}

/// Iterate over entities together with component flags.
struct EntityIter<'w> {
    inner: std::slice::Iter<'w, Entity>,
    // TODO
    flags: &'w [ComponentFlag],
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
        T::get(&self.components, entity)
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
        let entities = T::entities(&self.components).map(|c| EntityIter {
            inner: c,
            flags: &self.entities.component_flags,
        });
        Query {
            entities,
            components: &self.components,
            _marker: std::marker::PhantomData::default(),
        }
    }
    pub fn query_mut<'a, T>(&'a mut self) -> QueryMut<'a, T, CM>
    where
        T: FetchMut<'a, CM>,
    {
        let components = &raw mut self.components;
        let (state, entities) = unsafe { T::prepare(components, &self.entities) };
        QueryMut {
            entities,
            state,
            _marker: std::marker::PhantomData::default(),
        }
    }
}

pub unsafe trait ComponentHandler<T> {
    const MASK: u128;

    fn storage(&self) -> &ComponentStorage<T>;
    fn storage_mut(&mut self) -> &mut ComponentStorage<T>;
    unsafe fn storage_raw(c: *mut Self) -> *mut ComponentStorage<T>;
}
