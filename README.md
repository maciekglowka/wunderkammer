# Wunderkammer

[![crates.io](https://img.shields.io/crates/v/wunderkammer)](https://crates.io/crates/wunderkammer)
[![Documentation](https://img.shields.io/docsrs/wunderkammer)](https://docs.rs/wunderkammer/)
[![CI](https://github.com/maciekglowka/wunderkammer/actions/workflows/rust.yml/badge.svg)](https://github.com/maciekglowka/wunderkammer/actions/workflows/rust.yml)

**An experimental EC(S) crate.**

Uber-simple solutions for small-scoped, data-oriented games (e.g. roguelikes).

The crate does not enforce any specific game architecture.
It is meant to work well in a traditional game loop context.

Currently two independent functionalities are provided:
- an entity-component storage
- an event based scheduler queue

## Entity-Component Storage

Aims to solve the most basic requirements of an ECS storage:

- flexible object composition
- looking up entities with a required component set and processing their data

The crate relies entirely on static typing and compile-time checks, while still allowing
for runtime insertion and removal of components.

No unsafe code nor internal mutability (like `RefCell`) is used.
It won't crash on you if you'll try to borrow a component set mutably twice :)

The internal component storage is based on sparse set data structures, rather than archetypes.
It should still provide some level of cache locality
- the component data is held in contiguous vector types.

### Example EC usage

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

    let npcs = query!(world, With(health), Without(player)).collect::<Vec<_>>();
    assert_eq!(npcs.len(), 2);

    // apply poison
    query_execute!(world, With(health, poison), |_, h: &mut u32, _| {
        *h = h.saturating_sub(1);
    });

    assert_eq!(world.cmps.health.get(&player), Some(&4));
    assert_eq!(world.cmps.health.get(&rat), Some(&2));
    assert_eq!(world.cmps.health.get(&serpent), Some(&2));

    // heal the player
    let _ = world.cmps.poison.remove(player);
    let poisoned = query!(world, With(poison)).collect::<Vec<_>>();
    assert_eq!(poisoned.len(), 1);

    // use a resource
    world.res.current_level += 1;
}
```

## Event scheduler

The crate also provides a simple generic event queue / scheduler struct:

```rust ignore
// Events
struct Hit(Entity, i32); // (unit idx, dmg)
struct Kill(Entity); // unit idx

fn apply_damage(ev: &mut Hit, world: &mut World, cx: &mut SchedulerContext) -> EventResult {
    let health = world.cmps.health.get_mut(ev.0)
        .ok_or(EventError::Break)?;

    *health -= ev.1;
    if *health <= 0 {
        // Spawn a resulting event.
        cx.send(Kill(ev.0));
    }

    Ok(())
}
fn kill(ev: &mut Kill, world: &mut World) -> EventResult {
    world.despawn(ev.0);
    Ok(())
}

let mut scheduler = Scheduler::new();

scheduler.add_system(apply_damage);
scheduler.add_system(kill);

scheduler.send(Hit(0, 2));
scheduler.send(Hit(1, 2));
scheduler.send(Hit(2, 2));
```

Since handlers can be chained and events are passed as mutable references,
a higher priority (earlier) handler can modify the event data during execution:

```rust ignore
/// Executes before `apply_damage`
fn apply_shield(ev: &mut Hit, world: &mut World) -> EventResult {
    let shield = world.cmps.shield.get(cmd.0).ok_or(EventError::Continue)?;
    // Mutate the event, lower dmg by unit's shield value.
    ev.1 -= *shield;
    Ok(())
}
```

### Control flow

The `EventResult` return type allows for a basic control flow between the handlers:

- `Ok(())` -> uninterrupted execution
- `Err(CommandError::Break)` -> this event is invalid. Stop the execution of
    current and subsequent handlers.
- `Err(CommandError::Continue)` -> the current handler cannot process further.
    But do not stop execution of the next ones.

### Observability

Apart from the standard handlers, there is also a possiblity to create read-only observers.
They're mostly useful for decoupling parts of code that only need to react to finalized events
(like graphics, sound effects, journals).

```rust ignore
let log_observer = scheduler.observe::<Hit>();

let _ = std::thread::spawn(move || loop {
    if let Some(ev) = log_observer.next() {
        println!("{:?} got hit for {}", ev.0, ev.1);
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
});
```

## Crate goals

- Simple but flexible data storage for tiny games
- Reliability through compile-time checks and static typing
- Dynamic (runtime) component insertion and removal
- Recycling of despawned entities
- Flexible event queue (mostly for turn-based games and command patterns)
- Easy (de)serialization - via optional `serialize` feature
- Minimal dependencies
