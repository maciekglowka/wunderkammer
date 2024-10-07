pub(crate) mod components;
pub(crate) mod entity;
mod tests;

pub mod prelude {
    use super::*;
    pub use chamber_derive::Components;
    pub use components::{ComponentStorage, Components};
    pub use entity::{Entity, EntityStorage};
}
