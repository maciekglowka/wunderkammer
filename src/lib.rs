#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

#[cfg(feature = "events")]
pub mod events;
#[cfg(feature = "storage")]
pub mod storage;

pub mod prelude {
    use super::*;

    #[cfg(feature = "events")]
    pub use events::{BusHandle, EventDispatcher, EventSender, HandlerResult};

    #[cfg(feature = "storage")]
    pub use super::{insert, query, query_execute, query_iter};
    #[cfg(feature = "storage")]
    pub use storage::{
        components::{ComponentSet, ComponentStorage},
        entity::{Entity, EntityStorage},
        world::WorldStorage,
    };
    #[cfg(feature = "storage")]
    pub use wunderkammer_derive::ComponentSet;
}
