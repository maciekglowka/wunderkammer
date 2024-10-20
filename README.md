# Wunderkammer

An experimental ECS crate, with static component type definitions.

It aims to avoid any runtime checks (like interior mutability, dynamic casting), while still allowing to dynamically insert and remove entity's components.

The internal component storage is based on sparse set data structures, rather then archetypes.

## Example usage

```rust
use wunderkammer::prelude::*;

#[derive(ComponentSet, Default)]
struct Components {
    pub health: ComponentStorage<u32>,
    pub name: ComponentStorage<String>,
    pub player: ComponentStorage<()>, // marker component
    pub poison: ComponentStorage<()>,
    pub strength: ComponentStorage<u32>,
}

#[derive(Default)]
struct Resources {
    current_level: u32,
}

type World = WorldStorage<Components, Resources>;

fn main() {
        let mut world = World::default();

        let player = world.spawn();
        world.components.health.insert(player, 5);
        world.components.name.insert(player, "Player".to_string());
        world.components.player.insert(player, ());
        world.components.poison.insert(player, ());
        world.components.strength.insert(player, 3);

        let rat = world.spawn();
        world.components.health.insert(rat, 2);
        world.components.name.insert(rat, "Rat".to_string());
        world.components.strength.insert(rat, 1);

        let serpent = world.spawn();
        world.components.health.insert(serpent, 3);
        world.components.name.insert(serpent, "Serpent".to_string());
        world.components.poison.insert(serpent, ());
        world.components.strength.insert(serpent, 2);

        // find matching entities, returns HashSet<Entity>
        let npcs = query!(world, Without(player), With(health));
        assert_eq!(npcs.len(), 2);

        // apply poison
        query_execute_mut!(world, With(health, poison), |_, h: &mut u32, _| {
            *h = h.saturating_sub(1);
        });

        assert_eq!(world.components.health.get(player), Some(&4));
        assert_eq!(world.components.health.get(rat), Some(&2));
        assert_eq!(world.components.health.get(serpent), Some(&2));

        // heal player
        let _ = world.components.poison.remove(player);
        let poisoned = query!(world, With(poison));
        assert_eq!(poisoned.len(), 1);

        // use resource
        world.resources.current_level += 1;
    }
```