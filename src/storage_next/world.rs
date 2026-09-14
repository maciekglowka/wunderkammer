use super::components::ComponentStorage;
use super::entity::Entity;

pub trait Fetch {
    type Item;

    fn get(&self, entity: &Entity) -> Option<Self::Item>;
}
pub trait FetchMut<'w> {
    type Item;

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item>;
}

impl<'w, A> Fetch for &'w ComponentStorage<A> {
    type Item = &'w A;

    fn get(&self, entity: &Entity) -> Option<Self::Item> {
        ComponentStorage::get(self, entity)
    }
}
impl<'w, A: 'static> FetchMut<'w> for *mut ComponentStorage<A> {
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
impl<'w, A: FetchMut<'w>, B: FetchMut<'w>> FetchMut<'w> for (A, B) {
    type Item = (A::Item, B::Item);

    unsafe fn get_mut(&self, entity: &Entity) -> Option<Self::Item> {
        Some((self.0.get_mut(entity)?, self.1.get_mut(entity)?))
    }
}

pub struct Query<'a, F> {
    entities: std::slice::Iter<'a, Entity>,
    fetch: F,
}
impl<'a, F> Iterator for Query<'a, F>
where
    F: Fetch,
{
    type Item = F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.next() {
            if let Some(f) = self.fetch.get(entity) {
                return Some(f);
            }
        }
        None
    }
}

pub struct QueryMut<'a, F> {
    entities: std::slice::Iter<'a, Entity>,
    fetch: F,
}
impl<'a, F> Iterator for QueryMut<'a, F>
where
    F: FetchMut<'a>,
{
    type Item = F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.next() {
            if let Some(f) = unsafe { self.fetch.get_mut(entity) } {
                return Some(f);
            }
        }
        None
    }
}

pub trait IntoFetch<'a> {
    type Fetch: Fetch;
    type FetchMut: FetchMut<'a>;

    fn into(world: &'a World) -> Self::Fetch;
    fn into_mut(world: &'a mut World) -> Self::FetchMut;
    fn query(world: &'a World) -> Query<'a, Self::Fetch>;
    fn query_mut(world: &'a mut World) -> QueryMut<'a, Self::FetchMut>;
}

impl<'a, A> IntoFetch<'a> for A
where
    A: 'static,
    World: ComponentHandler<A>,
{
    type Fetch = &'a ComponentStorage<A>;
    type FetchMut = *mut ComponentStorage<A>;

    fn into(world: &'a World) -> Self::Fetch {
        <World as ComponentHandler<A>>::storage(world)
    }

    fn into_mut(world: &'a mut World) -> Self::FetchMut {
        <World as ComponentHandler<A>>::storage_raw(world)
    }

    fn query(world: &'a World) -> Query<'a, Self::Fetch> {
        let entities = <World as ComponentHandler<A>>::storage(world).entities();
        Query {
            entities,
            fetch: <Self as IntoFetch>::into(world),
        }
    }
    fn query_mut(world: &'a mut World) -> QueryMut<'a, Self::FetchMut> {
        let fetch = <Self as IntoFetch>::into_mut(world);
        let entities = <World as ComponentHandler<A>>::storage(world).entities();
        QueryMut { entities, fetch }
    }
}

impl<'a, A, B> IntoFetch<'a> for (A, B)
where
    A: 'static,
    B: 'static,
    World: ComponentHandler<A>,
    World: ComponentHandler<B>,
{
    type Fetch = (&'a ComponentStorage<A>, &'a ComponentStorage<B>);
    type FetchMut = (*mut ComponentStorage<A>, *mut ComponentStorage<B>);

    fn into(world: &'a World) -> Self::Fetch {
        (
            <World as ComponentHandler<A>>::storage(world),
            <World as ComponentHandler<B>>::storage(world),
        )
    }
    fn into_mut(world: &'a mut World) -> Self::FetchMut {
        const {
            if 1 << <World as ComponentHandler<A>>::MASK & 1 << <World as ComponentHandler<B>>::MASK
                != 0
            {
                panic!("Duplicated query type");
            }
        }
        (
            <World as ComponentHandler<A>>::storage_raw(world),
            <World as ComponentHandler<B>>::storage_raw(world),
        )
    }
    fn query(world: &'a World) -> Query<'a, Self::Fetch> {
        let entities = <World as ComponentHandler<A>>::storage(world).entities();
        Query {
            entities,
            fetch: <Self as IntoFetch>::into(world),
        }
    }
    fn query_mut(world: &'a mut World) -> QueryMut<'a, Self::FetchMut> {
        let fetch = <Self as IntoFetch>::into_mut(world);
        let entities = <World as ComponentHandler<A>>::storage(world).entities();
        QueryMut { entities, fetch }
    }
}

pub struct World {
    entities: super::entity::EntityStorage,
    position: ComponentStorage<u32>,
    velocity: ComponentStorage<i32>,
    alive: ComponentStorage<bool>,
}
impl World {
    pub fn new() -> Self {
        Self {
            entities: super::entity::EntityStorage::default(),
            position: ComponentStorage::default(),
            velocity: ComponentStorage::default(),
            alive: ComponentStorage::default(),
        }
    }
    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }
    pub fn insert<T>(&mut self, entity: Entity, value: T)
    where
        Self: ComponentHandler<T>,
    {
        self.storage_mut().__insert(entity, value);
    }
    pub fn get<'a, T>(&'a self, entity: &Entity) -> Option<<T::Fetch as Fetch>::Item>
    where
        T: IntoFetch<'a>,
    {
        T::into(self).get(entity)
    }
    pub fn get_mut<'a, T>(&'a mut self, entity: &Entity) -> Option<<T::FetchMut as FetchMut>::Item>
    where
        T: IntoFetch<'a>,
    {
        unsafe { T::into_mut(self).get_mut(entity) }
    }
    pub fn query<'a, T>(&'a self) -> Query<'a, T::Fetch>
    where
        T: IntoFetch<'a>,
    {
        T::query(self)
    }
    pub fn query_mut<'a, T>(&'a mut self) -> QueryMut<'a, T::FetchMut>
    where
        T: IntoFetch<'a>,
    {
        T::query_mut(self)
    }
}

trait ComponentHandler<T> {
    const MASK: u128 = 0;
    fn storage(&self) -> &ComponentStorage<T>;
    fn storage_mut(&mut self) -> &mut ComponentStorage<T>;
    fn storage_raw(&mut self) -> *mut ComponentStorage<T>;
}

impl ComponentHandler<u32> for World {
    const MASK: u128 = 1;

    fn storage(&self) -> &ComponentStorage<u32> {
        &self.position
    }
    fn storage_mut(&mut self) -> &mut ComponentStorage<u32> {
        &mut self.position
    }
    fn storage_raw(&mut self) -> *mut ComponentStorage<u32> {
        &raw mut self.position
    }
}
impl ComponentHandler<i32> for World {
    const MASK: u128 = 2;

    fn storage(&self) -> &ComponentStorage<i32> {
        &self.velocity
    }
    fn storage_mut(&mut self) -> &mut ComponentStorage<i32> {
        &mut self.velocity
    }
    fn storage_raw(&mut self) -> *mut ComponentStorage<i32> {
        &raw mut self.velocity
    }
}
impl ComponentHandler<bool> for World {
    const MASK: u128 = 3;

    fn storage(&self) -> &ComponentStorage<bool> {
        &self.alive
    }
    fn storage_mut(&mut self) -> &mut ComponentStorage<bool> {
        &mut self.alive
    }
    fn storage_raw(&mut self) -> *mut ComponentStorage<bool> {
        &raw mut self.alive
    }
}
