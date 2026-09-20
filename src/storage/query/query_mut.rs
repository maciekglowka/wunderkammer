use crate::storage::components::get_dense_index;
use crate::storage::entity::{Entity, EntityStorage, IdSize};
use crate::storage::ComponentHandler;

use super::EntityIter;

pub struct QueryMut<'w, F, CM>
where
    F: FetchMut<'w, CM>,
{
    state: F::View,
    entities: Option<EntityIter<'w>>,
    _marker: std::marker::PhantomData<(F, &'w CM)>,
}
impl<'w, F, CM> QueryMut<'w, F, CM>
where
    F: FetchMut<'w, CM>,
{
    pub(crate) fn new(state: F::View, entities: Option<EntityIter<'w>>) -> Self {
        Self {
            state,
            entities,
            _marker: std::marker::PhantomData,
        }
    }
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

pub struct ViewMut<'w, A> {
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

pub unsafe trait FetchMut<'w, CM>
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
