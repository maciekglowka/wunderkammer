#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

#[cfg(feature = "events")]
pub mod events;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "storage")]
pub use wunderkammer_derive::storage;

#[cfg(feature = "storage")]
pub use storage::Entity;

pub mod prelude {
    use super::*;

    #[cfg(feature = "events")]
    pub use events::{BusHandle, EventDispatcher, EventSender, HandlerResult};

    #[cfg(feature = "storage")]
    pub use wunderkammer_derive::storage;
}
