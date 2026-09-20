mod query_mut;
mod query_ref;

pub use query_mut::QueryMut;
pub use query_ref::Query;

pub(crate) use query_mut::FetchMut;
pub(crate) use query_ref::Fetch;

use super::entity::ComponentFlag;
use super::Entity;

/// Iterate over entities together with component flags.
pub struct EntityIter<'w> {
    pub(crate) inner: std::slice::Iter<'w, Entity>,
    // TODO
    pub(crate) flags: &'w [ComponentFlag],
}
impl<'w> Iterator for EntityIter<'w> {
    type Item = (&'w Entity, ComponentFlag);

    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.inner.next()?;
        let flag = self.flags[entity.id as usize];
        Some((entity, flag))
    }
}
