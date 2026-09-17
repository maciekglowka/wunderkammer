use crate::storage_next::entity::{ComponentFlag, EntityStorage};

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
    const COLLISION_MASK: u128;
    unsafe fn get_mut(components: &'w CM, entity: &Entity) -> Option<Self>;

    fn entities(_components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
        None
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for &'w A
where
    CM: ComponentHandler<A>,
{
    const COLLISION_MASK: u128 = <CM as ComponentHandler<A>>::MASK;

    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        const {
            assert!(
                <Self as FetchMut<'w, CM>>::COLLISION_MASK != 0,
                "component collision mask can't be 0"
            )
        };
        components.storage().get(entity)
    }
    fn entities(components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
        Some(components.storage().entities())
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for Option<&'w A>
where
    CM: ComponentHandler<A>,
{
    const COLLISION_MASK: u128 = <CM as ComponentHandler<A>>::MASK;

    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        const {
            assert!(
                <Self as FetchMut<'w, CM>>::COLLISION_MASK != 0,
                "component collision mask can't be 0"
            )
        };
        Some(components.storage().get(entity))
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for &'w mut A
where
    CM: ComponentHandler<A>,
{
    const COLLISION_MASK: u128 = <CM as ComponentHandler<A>>::MASK;

    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        const {
            assert!(
                <Self as FetchMut<'w, CM>>::COLLISION_MASK != 0,
                "component collision mask can't be 0"
            )
        };
        components.storage_mut().get_mut(entity)
    }
    fn entities(components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
        Some(components.storage().entities())
    }
}
unsafe impl<'w, A, CM> FetchMut<'w, CM> for Option<&'w mut A>
where
    CM: ComponentHandler<A>,
{
    const COLLISION_MASK: u128 = <CM as ComponentHandler<A>>::MASK;

    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        const {
            assert!(
                <Self as FetchMut<'w, CM>>::COLLISION_MASK != 0,
                "component collision mask can't be 0"
            )
        };
        Some(components.storage_mut().get_mut(entity))
    }
}
unsafe impl<'w, CM> FetchMut<'w, CM> for Entity {
    const COLLISION_MASK: u128 = 0;

    unsafe fn get_mut(_components: &'w mut CM, entity: &Entity) -> Option<Self> {
        Some(*entity)
    }
}

unsafe impl<'w, A, B, CM> FetchMut<'w, CM> for (A, B)
where
    A: FetchMut<'w, CM>,
    B: FetchMut<'w, CM>,
{
    const COLLISION_MASK: u128 = A::COLLISION_MASK | B::COLLISION_MASK;

    unsafe fn get_mut(components: &'w mut CM, entity: &Entity) -> Option<Self> {
        const {
            assert!(
                A::COLLISION_MASK & B::COLLISION_MASK == 0,
                "conflicting query type"
            );
        };

        let components = &raw mut *components;
        Some((
            A::get_mut(components.as_mut_unchecked(), entity)?,
            B::get_mut(components.as_mut_unchecked(), entity)?,
        ))
    }
    fn entities(components: &'w CM) -> Option<std::slice::Iter<'w, Entity>> {
        A::entities(components).or_else(|| B::entities(components))
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

pub struct QueryMut<'w, F, CM>
where
    F: FetchMut<'w, CM>,
{
    entities: EntityIter<'w>,
    components: *mut CM,
    _marker: std::marker::PhantomData<F>,
}
impl<'w, F, CM: 'w> Iterator for QueryMut<'w, F, CM>
where
    F: FetchMut<'w, CM>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity, flags)) = self.entities.next() {
            if let Some(f) = unsafe { F::get_mut(self.components.as_mut_unchecked(), entity) } {
                return Some(f);
            }
        }
        None
    }
}

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
    pub fn query_mut<'a, T>(&'a mut self) -> QueryMut<'a, T, CM>
    where
        T: FetchMut<'a, CM>,
    {
        let components = &raw mut self.components;
        QueryMut {
            entities: EntityIter {
                inner: T::entities(&self.components),
                flags: &self.entities.component_flags,
            },
            components,
            _marker: std::marker::PhantomData::default(),
        }
    }
}

pub trait ComponentHandler<T> {
    const MASK: u128;

    fn storage(&self) -> &ComponentStorage<T>;
    fn storage_mut(&mut self) -> &mut ComponentStorage<T>;
    fn storage_raw(&mut self) -> *mut ComponentStorage<T>;
}
