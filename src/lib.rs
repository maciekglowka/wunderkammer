pub(crate) mod components;
pub(crate) mod entity;
pub(crate) mod query;
mod tests;
pub(crate) mod world;

pub mod prelude {
    use super::*;
    pub use super::{query, query_execute, query_execute_mut};
    pub use chamber_derive::Components;
    pub use components::{ComponentStorage, Components};
    pub use entity::{Entity, EntityStorage};
    pub use world::WorldStorage;
}
