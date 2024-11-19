# Wunderkammer

[![crates.io](https://img.shields.io/crates/v/wunderkammer)](https://crates.io/crates/wunderkammer)
[![Documentation](https://img.shields.io/docsrs/wunderkammer)](https://docs.rs/wunderkammer/)
[![CI](https://github.com/maciekglowka/wunderkammer/actions/workflows/rust.yml/badge.svg)](https://github.com/maciekglowka/wunderkammer/actions/workflows/rust.yml)

**An experimental EC(S) crate.**

Provides a simple Entity-Component structure, meant for small scoped data oriented games (eg. roguelikes).


It aims to solve the most basic requirements of a component storage:

- flexible object composition 
- querying for entities with a certain component set attached and processing their data

The crate is merely a storage data structure and does not enforce any specific game architecture.
It is meant to also work in a traditional game loop context.

Relies completely on static typing and compile time checks, while still allowing
for runtime insertion and removal of components.
    
No unsafe code, internal mutability (like `RefCell`) or dynamic typing
is used. It won't crash on you if you'll try to borrow a component set mutably twice :)

The internal component storage is based on sparse set data structures, rather then archetypes.
It should still provide some level of cache locality - the component data is held in continuous vector types.

## Crate goals

- Simple but flexible data storage for tiny games
- Reliability through compile-time checks and static typing
- Dynamic component insertion and removal
- Recycling of despawned entities
- Easy (de)serialization - via optional `serialize` feature
- As few dependencies as possible (currently only `syn` and `quote` libs to handle derive macros, + `serde` behind a feature flag)

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

        // spawn player
        let player = world.spawn();
        world.components.health.insert(player, 5);
        world.components.name.insert(player, "Player".to_string());
        world.components.player.insert(player, ());
        world.components.strength.insert(player, 3);

        // spawn npcs
        let rat = world.spawn();
        world.components.health.insert(rat, 2);
        world.components.name.insert(rat, "Rat".to_string());
        world.components.strength.insert(rat, 1);

        let serpent = world.spawn();
        world.components.health.insert(serpent, 3);
        world.components.name.insert(serpent, "Serpent".to_string());
        world.components.strength.insert(serpent, 2);

        // find all npc entities, returns HashSet<Entity>
        let npcs = query!(world, Without(player), With(health));
        assert_eq!(npcs.len(), 2);

        // poison the player and the serpent
        world.components.poison.insert(player, ());
        world.components.poison.insert(serpent, ());

        // apply poison damage
        query_execute_mut!(world, With(health, poison), |_, h: &mut u32, _| {
            *h = h.saturating_sub(1);
        });

        assert_eq!(world.components.health.get(player), Some(&4));
        assert_eq!(world.components.health.get(rat), Some(&2));
        assert_eq!(world.components.health.get(serpent), Some(&2));

        // heal player from poison
        let _ = world.components.poison.remove(player);
        let poisoned = query!(world, With(poison));
        assert_eq!(poisoned.len(), 1);

        // use resource
        world.resources.current_level += 1;
    }
```
