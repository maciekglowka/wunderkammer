use wunderkammer::prelude::*;

struct Unit {
    health: i32,
    invincible: bool,
}

// Simplified world struct for brevity.
// Typically a entity-component storage would be used.
struct World {
    units: Vec<Unit>,
}

// Events
struct Hit(usize, i32); // (unit idx, dmg)

// Handlers
fn validate_hit(ev: &mut Hit, world: &mut World) -> EventResult {
    // If the unit is invincible, break this event flow
    match world.units[ev.0].invincible {
        true => Err(EventError::Break),
        false => Ok(()),
    }
}
fn apply_damage(ev: &mut Hit, world: &mut World) -> EventResult {
    world.units[ev.0].health -= ev.1;
    Ok(())
}

fn main() {
    let a = Unit {
        health: 3,
        invincible: false,
    };
    let b = Unit {
        health: 2,
        invincible: true,
    };
    let mut world = World { units: vec![a, b] };

    let mut scheduler = Scheduler::new();

    scheduler.add_system_with_priority(validate_hit, 0);
    scheduler.add_system_with_priority(apply_damage, 1);

    scheduler.send(Hit(0, 2));
    scheduler.send(Hit(1, 2));

    while scheduler.step(&mut world) {}

    assert_eq!(world.units[0].health, 1);
    assert_eq!(world.units[1].health, 2);
}
