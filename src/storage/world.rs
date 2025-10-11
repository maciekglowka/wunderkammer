#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use super::components::ComponentSet;
use super::entity::{Entity, EntityStorage};

/// Main storage struct responsible for tracking entities, components and
/// resources.
#[derive(Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct WorldStorage<C, R> {
    entities: EntityStorage,
    pub cmps: C,
    pub res: R,
}
impl<C: ComponentSet, R: Default> WorldStorage<C, R> {
    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }
    pub fn despawn(&mut self, entity: Entity) {
        self.cmps.remove_all_components(entity);
        self.entities.despawn(entity);
    }
    pub fn is_valid(&self, entity: &Entity) -> bool {
        self.entities.is_valid(entity)
    }
    pub fn entities(&self) -> impl Iterator<Item = &Entity> + use<'_, C, R> {
        self.entities.all()
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::prelude::*;

    #[cfg(feature = "serialize")]
    use serde::{Deserialize, Serialize};

    #[cfg(feature = "serialize")]
    #[test]
    fn serialize() {
        #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
        struct Position {
            x: u32,
            y: u32,
        };

        #[derive(ComponentSet, Default, Serialize, Deserialize)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
            pub position: ComponentStorage<Position>,
        }
        #[derive(Default, Serialize, Deserialize)]
        struct R {
            globals: Vec<String>,
        }
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();

        insert!(w, health, a, 15);
        insert!(w, position, a, Position { x: 2, y: 5 });
        insert!(w, name, a, "Fifteen".to_string());

        insert!(w, health, b, 20);
        insert!(w, position, b, Position { x: 5, y: 4 });

        w.res.globals.push("GlobalTwenty".to_string());

        let serialized = serde_json::to_string(&w).unwrap();

        let w_deserialized: WorldStorage<C, R> = serde_json::from_str(&serialized).unwrap();

        let entities = query!(w_deserialized, With(health, name))
            .copied()
            .collect::<Vec<_>>();

        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&a));
        assert_eq!(
            *w_deserialized.cmps.position.get(&a).unwrap(),
            Position { x: 2, y: 5 }
        );

        assert!(w_deserialized
            .res
            .globals
            .contains(&"GlobalTwenty".to_string()));
    }
}
