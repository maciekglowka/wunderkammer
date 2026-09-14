use super::components::ComponentStorage;
use super::entity::Entity;

// pub trait Fetch {
//     type Item<'a>
//     where
//         Self: 'a;

//     // fn get<'a>(&'a self, world: &'a World, entity: &Entity) ->
//     fn get<'a>(&'a self, entity: &Entity) -> Option<Self::Item<'a>>;
// }
// impl<A> Fetch for ComponentStorage<A> {
//     type Item<'a>
//         = &'a A
//     where
//         Self: 'a;

//     fn get<'a>(&'a self, entity: &Entity) -> Option<Self::Item<'a>> {
//         self.get(entity)
//     }
// }
// impl<A, B> Fetch for (ComponentStorage<A>, ComponentStorage<B>) {
//     type Item<'a>
//         = (&'a A, &'a B)
//     where
//         Self: 'a;

//     fn get<'a>(&'a self, entity: &Entity) -> Option<Self::Item<'a>> {
//         Some((self.0.get(entity)?, self.1.get(entity)?))
//     }
// }

pub trait Fetch {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

impl<A> Fetch for (std::slice::Iter<'_, Entity>, &ComponentStorage<A>) {
    type Item<'a>
        = &'a A
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        self.1.get(self.0.next()?)
    }
}
impl<A, B> Fetch
    for (
        std::slice::Iter<'_, Entity>,
        &ComponentStorage<A>,
        &ComponentStorage<B>,
    )
{
    type Item<'a>
        = (&'a A, &'a B)
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        let e = self.0.next()?;
        Some((self.1.get(e)?, self.2.get(e)?))
    }
}

// impl<A> Fetch for A
// where
//     World: ComponentHandler<A>,
// {
//     type Item<'a>
//         = &'a A
//     where
//         Self: 'a;

//     fn get<'a>(&'a self, world: &'a World, entity: &Entity) -> Self::Item<'a>
// {         world.get(entity).unwrap()
//     }
// }
// impl<A, B> Fetch for (A, B)
// where
//     World: ComponentHandler<A>,
//     World: ComponentHandler<B>,
// {
//     type Item<'a>
//         = (&'a A, &'a B)
//     where
//         Self: 'a;

//     fn get<'a>(&'a self, world: &'a World, entity: &Entity) -> Self::Item<'a>
// {         (world.get(entity).unwrap(), world.get(entity).unwrap())
//     }
// }

// struct Query<'a, T: Fetch> {

// }

// trait IntoQuery<T> {
//     type Item<'a>
//     where
//         Self: 'a;

//     fn next<'a>(&mut self) -> Option<Self::Item<'a>>;
// }

// impl<A> IntoQuery<A> for World
// where
//     World: ComponentHandler<A>,
// {
//     type Item<'a>
//         = &'a A
//     where
//         Self: 'a;

//     fn next<'a>(&mut self) -> Option<Self::Item<'a>> {
//         //
//     }
// }

// struct Query<T: Fetch>

// struct Query<'a, F>
// where
//     F: Fetch,
// {
//     entities: std::slice::Iter<'a, Entity>,
//     fetch: F,
// }
// impl<'a, F> Query<'a, F>
// where
//     F: Fetch,
// {
//     fn next(&'a mut self) -> Option<F::Item<'a>> {
//         while let Some(entity) = self.entities.next() {
//             if let Some(item) = self.fetch.get(entity) {
//                 return Some(item);
//             }
//         }
//         None
//     }
// }
//
pub struct Query<T>(T);
impl<T> Query<T>
where
    T: Fetch,
{
    pub fn next<'a>(&'a mut self) -> Option<T::Item<'a>> {
        self.0.next()
    }
}

pub trait IntoQuery<'a> {
    type Q: Fetch;

    fn get(world: &'a World) -> Query<Self::Q>;
}

impl<'a, A> IntoQuery<'a> for A
where
    A: 'static,
    World: ComponentHandler<A>,
{
    type Q = (std::slice::Iter<'a, Entity>, &'a ComponentStorage<A>);

    fn get(world: &'a World) -> Query<Self::Q> {
        Query((world.storage().entities(), world.storage()))
    }
}
impl<'a, A, B> IntoQuery<'a> for (A, B)
where
    A: 'static,
    B: 'static,
    World: ComponentHandler<A>,
    World: ComponentHandler<B>,
{
    type Q = (
        std::slice::Iter<'a, Entity>,
        &'a ComponentStorage<A>,
        &'a ComponentStorage<B>,
    );

    fn get(world: &'a World) -> Query<Self::Q> {
        Query((
            <World as ComponentHandler<A>>::storage(world).entities(),
            world.storage(),
            world.storage(),
        ))
    }
}

fn query_position(world: &World) {
    world.position.entities();
}

fn query_position_velocity(world: &World) {
    world
        .position
        .entities()
        .filter(|e| world.velocity.get(e).is_some());
}

pub struct World {
    entities: super::entity::EntityStorage,
    position: ComponentStorage<u32>,
    velocity: ComponentStorage<i32>,
    alive: ComponentStorage<bool>,
}
impl World {
    // pub fn query<'a, T, Q>(&'a self) -> Query<Q>
    // where
    //     Q: Fetch,
    //     T: IntoQuery<'a, Q = Q>,
    // {
    //     T::get(self)
    // }
    pub fn query<'a, T>(&'a self) -> Query<T::Q>
    where
        T: IntoQuery<'a>,
    {
        T::get(self)
    }
    pub fn query2<T, U>(&self)
    where
        Self: ComponentHandler<T>,
        Self: ComponentHandler<U>,
        T: 'static,
        U: 'static,
    {
        const {
            if 1 << <World as ComponentHandler<T>>::MASK & 1 << <World as ComponentHandler<U>>::MASK
                != 0
            {
                panic!("Duplicated query type");
            }
        }
    }
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
    pub fn get<T>(&self, entity: &Entity) -> Option<&T>
    where
        Self: ComponentHandler<T>,
    {
        self.storage().get(entity)
    }
    pub fn get_mut<T>(&mut self, entity: &Entity) -> Option<&mut T>
    where
        Self: ComponentHandler<T>,
    {
        self.storage_mut().get_mut(entity)
    }
}

trait ComponentHandler<T> {
    const MASK: u128 = 0;
    fn storage(&self) -> &ComponentStorage<T>;
    fn storage_mut(&mut self) -> &mut ComponentStorage<T>;
}

impl ComponentHandler<u32> for World {
    const MASK: u128 = 1;

    fn storage(&self) -> &ComponentStorage<u32> {
        &self.position
    }
    fn storage_mut(&mut self) -> &mut ComponentStorage<u32> {
        &mut self.position
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
}
impl ComponentHandler<bool> for World {
    const MASK: u128 = 3;

    fn storage(&self) -> &ComponentStorage<bool> {
        &self.alive
    }
    fn storage_mut(&mut self) -> &mut ComponentStorage<bool> {
        &mut self.alive
    }
}
