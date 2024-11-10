pub(crate) mod components;
pub(crate) mod entity;
pub(crate) mod query;
mod tests;
pub(crate) mod world;

pub mod prelude {
    use super::*;
    pub use super::{query, query_execute, query_execute_mut, query_iter};
    pub use components::{ComponentSet, ComponentStorage};
    pub use entity::{Entity, EntityStorage};
    pub use world::WorldStorage;
    pub use wunderkammer_derive::ComponentSet;
}
