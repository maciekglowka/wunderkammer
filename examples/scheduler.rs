use wunderkammer::prelude::*;

struct Unit {
    health: i32,
    shield: i32,
    alive: bool,
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
fn apply_shield(ev: &mut Hit, world: &mut World) -> EventResult {
    // Mutate the event, lower dmg by unit's shield value.
    ev.1 -= world.units[ev.0].shield;
    Ok(())
}
fn apply_damage(ev: &mut Hit, world: &mut World, cx: &mut SchedulerContext) -> EventResult {
    world.units[ev.0].health -= ev.1;
    if world.units[ev.0].health == 0 {
        // Spawn a resulting ev.
        cx.send(Kill(ev.0));
    }
    Ok(())
}
fn kill(ev: &mut Kill, world: &mut World) -> EventResult {
    world.units[ev.0].alive = false;
    Ok(())
}

fn main() {
    let a = Unit {
        health: 3,
        shield: 1,
        alive: true,
    };
    let b = Unit {
        health: 2,
        shield: 0,
        alive: true,
    };
    let mut world = World { units: vec![a, b] };

    let mut scheduler = Scheduler::new();

    scheduler.add_system_with_priority(apply_shield, 0);
    scheduler.add_system_with_priority(apply_damage, 1);
    scheduler.add_system(kill);

    scheduler.send(Hit(0, 2));
    scheduler.send(Hit(1, 2));

    while scheduler.step(&mut world) {}

    assert_eq!(world.units[0].health, 2);
    assert_eq!(world.units[1].health, 0);
    assert!(!world.units[1].alive);
}
