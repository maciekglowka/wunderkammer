use wunderkammer::prelude::*;

struct Unit {
    alive: bool,
    health: i32,
    invincible: bool,
    shield: Option<i32>,
}

// Simplified world struct for brevity.
// Typically a entity-component storage would be used.
struct World {
    units: Vec<Unit>,
}

// Events
struct Hit(usize, i32); // (unit idx, dmg)
struct Kill(usize); // unit idx

// Handlers
fn check_invincible(ev: &mut Hit, world: &mut World) -> EventResult {
    // If the unit is invincible, break this event flow
    match world.units[ev.0].invincible {
        true => Err(EventError::Break),
        false => Ok(()),
    }
}
fn apply_shield(ev: &mut Hit, world: &mut World) -> EventResult {
    // Bail if the unit does not have shield.
    // However, let the flow continue,
    // as damage can still be dealt correctly.
    let shield = world.units[ev.0].shield.ok_or(EventError::Continue)?;

    // Mutate the event, lower dmg by unit's shield value.
    ev.1 -= shield;
    Ok(())
}
fn apply_damage(ev: &mut Hit, world: &mut World, cx: &mut SchedulerContext) -> EventResult {
    world.units[ev.0].health -= ev.1;

    if world.units[ev.0].health <= 0 {
        // Spawn a resulting event.
        cx.send_immediate(Kill(ev.0));
    }

    Ok(())
}
fn kill(ev: &mut Kill, world: &mut World) -> EventResult {
    world.units[ev.0].alive = false;
    Ok(())
}

fn main() {
    let a = Unit {
        alive: true,
        health: 2,
        invincible: false,
        shield: None,
    };
    let b = Unit {
        alive: true,
        health: 2,
        invincible: true,
        shield: None,
    };
    let c = Unit {
        alive: true,
        health: 2,
        invincible: false,
        shield: Some(1),
    };

    let mut world = World {
        units: vec![a, b, c],
    };
    let mut scheduler = Scheduler::new();

    scheduler.add_system_with_priority(check_invincible, 0);
    scheduler.add_system_with_priority(apply_shield, 1);
    scheduler.add_system_with_priority(apply_damage, 2);
    scheduler.add_system(kill);

    scheduler.send(Hit(0, 2));
    scheduler.send(Hit(1, 2));
    scheduler.send(Hit(2, 2));

    // Process all the events
    while scheduler.step(&mut world) {}

    assert_eq!(world.units[0].health, 0);
    assert_eq!(world.units[1].health, 2);
    assert_eq!(world.units[2].health, 1);

    assert!(!world.units[0].alive);
}
