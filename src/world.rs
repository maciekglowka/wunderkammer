#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::components::ComponentSet;
use crate::entity::{Entity, EntityStorage};

/// Main storage struct responsible for tracking entities, components and
/// resources.
#[derive(Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct WorldStorage<C, R> {
    entities: EntityStorage,
    pub components: C,
    pub resources: R,
}
impl<C: ComponentSet, R: Default> WorldStorage<C, R> {
    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }
    pub fn despawn(&mut self, entity: Entity) {
        self.components.despawn(entity);
        self.entities.despawn(entity);
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

        w.components.health.insert(a, 15);
        w.components.position.insert(a, Position { x: 2, y: 5 });
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 20);
        w.components.position.insert(b, Position { x: 5, y: 4 });

        w.resources.globals.push("GlobalTwenty".to_string());

        let serialized = serde_json::to_string(&w).unwrap();

        let w_deserialized: WorldStorage<C, R> = serde_json::from_str(&serialized).unwrap();

        let entities = query!(w_deserialized, With(health, name));
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&a));
        assert_eq!(
            *w_deserialized.components.position.get(a).unwrap(),
            Position { x: 2, y: 5 }
        );

        assert!(w_deserialized
            .resources
            .globals
            .contains(&"GlobalTwenty".to_string()));
    }
}
