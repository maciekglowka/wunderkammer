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

    let player = world.spawn();
    insert!(world, health, player, 5);
    insert!(world, name, player, "Player".to_string());
    insert!(world, player, player, ());
    insert!(world, poison, player, ());
    insert!(world, strength, player, 3);

    let rat = world.spawn();
    insert!(world, health, rat, 2);
    insert!(world, name, rat, "Rat".to_string());
    insert!(world, strength, rat, 1);

    let serpent = world.spawn();
    insert!(world, health, serpent, 3);
    insert!(world, name, serpent, "Serpent".to_string());
    insert!(world, poison, serpent, ());
    insert!(world, strength, serpent, 2);

    // find matching entities, returns HashSet<Entity>
    let npcs = query!(world, With(health), Without(player)).collect::<Vec<_>>();
    assert_eq!(npcs.len(), 2);

    // apply poison
    query_execute!(world, With(health, poison), |_, h: &mut u32, _| {
        *h = h.saturating_sub(1);
    });

    assert_eq!(world.cmps.health.get(&player), Some(&4));
    assert_eq!(world.cmps.health.get(&rat), Some(&2));
    assert_eq!(world.cmps.health.get(&serpent), Some(&2));

    // heal player
    let _ = world.cmps.poison.remove(player);
    let poisoned = query!(world, With(poison)).collect::<Vec<_>>();
    assert_eq!(poisoned.len(), 1);

    // use resource
    world.res.current_level += 1;
}
```
