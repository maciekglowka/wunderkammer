pub(crate) mod components;
pub(crate) mod entity;

pub mod prelude {
    use super::*;
    pub use entity::{Entity, EntityStorage};
}
