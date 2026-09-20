use crate::ComponentStorage;

use super::entity::{Entity, EntityStorage};
use super::storage::{ComponentHandler, EntityIter};

pub struct Query<'w, F, CM>
where
    F: Fetch<'w, CM>,
{
    state: F::View,
    entities: Option<EntityIter<'w>>,
    // pub(crate) components: &'w CM,
    _marker: std::marker::PhantomData<F>,
}
impl<'w, F, CM> Query<'w, F, CM>
where
    F: Fetch<'w, CM>,
{
    pub(crate) fn new(state: F::View, entities: Option<EntityIter<'w>>) -> Self {
        Self {
            state,
            entities,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<'w, F, CM> Iterator for Query<'w, F, CM>
where
    F: Fetch<'w, CM>,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity, flags)) = self.entities.as_mut()?.next() {
            if let Some(f) = F::get(&self.state, entity) {
                return Some(f);
            }
        }
        None
    }
}

pub(crate) struct View<'w, A> {
    inner: &'w ComponentStorage<A>,
}
impl<'w, A> View<'w, A> {
    fn get(&self, entity: &Entity) -> Option<&'w A> {
        self.inner.get(entity)
    }
}

pub(crate) trait Fetch<'w, CM>
where
    Self: Sized,
{
    type View;

    fn prepare(
        components: &'w CM,
        entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter<'w>>);
    fn get(view: &Self::View, entity: &Entity) -> Option<Self>;
}

impl<'w, A, CM> Fetch<'w, CM> for &'w A
where
    CM: ComponentHandler<A>,
{
    type View = View<'w, A>;

    fn prepare(
        components: &'w CM,
        entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter<'w>>) {
        let storage = components.storage();
        (
            View { inner: storage },
            Some(EntityIter {
                inner: storage.entities(),
                flags: &entity_storage.component_flags,
            }),
        )
    }
    fn get(view: &Self::View, entity: &Entity) -> Option<Self> {
        view.get(entity)
    }
}
impl<'w, A, CM> Fetch<'w, CM> for Option<&'w A>
where
    CM: ComponentHandler<A>,
{
    type View = View<'w, A>;

    fn prepare(
        components: &'w CM,
        _entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter<'w>>) {
        let storage = components.storage();
        (View { inner: storage }, None)
    }
    fn get(view: &Self::View, entity: &Entity) -> Option<Self> {
        Some(view.get(entity))
    }
}
impl<'w, CM> Fetch<'w, CM> for Entity {
    type View = ();

    fn prepare(
        _components: &'w CM,
        _entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter<'w>>) {
        ((), None)
    }
    fn get(_view: &Self::View, entity: &Entity) -> Option<Self> {
        Some(*entity)
    }
}

impl<'w, A, B, CM> Fetch<'w, CM> for (A, B)
where
    A: Fetch<'w, CM>,
    B: Fetch<'w, CM>,
{
    type View = (A::View, B::View);

    fn prepare(
        components: &'w CM,
        entity_storage: &'w EntityStorage,
    ) -> (Self::View, Option<EntityIter<'w>>) {
        let (ca, ea) = A::prepare(components, entity_storage);
        let (cb, eb) = B::prepare(components, entity_storage);
        let entities = ea.or_else(|| eb);
        ((ca, cb), entities)
    }
    fn get(view: &Self::View, entity: &Entity) -> Option<Self> {
        let (va, vb) = view;
        Some((A::get(va, entity)?, B::get(vb, entity)?))
    }
}
