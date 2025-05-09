#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
pub(crate) mod components;
pub(crate) mod entity;
pub mod markers;
pub(crate) mod observer;
pub(crate) mod query;
pub(crate) mod scheduler;
mod tests;
pub(crate) mod utils;
pub(crate) mod world;

pub mod prelude {
    use super::*;
    pub use super::{insert, query, query_execute, query_execute_mut, query_iter};
    pub use components::{ComponentSet, ComponentStorage};
    pub use entity::{Entity, EntityStorage};
    pub use observer::{ObservableQueue, Observer};
    pub use scheduler::{CommandError, CommandHandler, Scheduler, SchedulerContext};
    pub use world::WorldStorage;
    pub use wunderkammer_derive::ComponentSet;
}
