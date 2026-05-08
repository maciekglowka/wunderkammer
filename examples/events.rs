use wunderkammer::events::markers::Mut;
use wunderkammer::events::EventDispatcher;
use wunderkammer::prelude::{event_bus, EventSender, EventSubscriber};

struct World(u32);
struct Graphics(u32);

fn main() {
    let mut world = World(0);
    let mut graphics = Graphics(0);
    let mut bus: EventSubscriber<World> = event_bus();
    let mut g_bus: EventSubscriber<(Graphics, World), (Mut, Mut)> = bus.spawn_subscriber();

    let handler = |ev: &u32, w: &mut World| {
        println!("W: {ev}");
        Ok(())
    };
    let g_handler = |ev: &u32, w: (&mut Graphics, &mut World)| {
        w.0 .0 += 1;
        println!("G: {ev} {}", w.0 .0);
        Ok(())
    };
    let sender = |ev: &u32, w: &mut World, s: &mut EventSender| {
        println!("S: {ev}");
        s.send(8u32);
        Ok(())
    };

    bus.add_handler(handler);
    bus.add_handler(sender);
    g_bus.add_handler(g_handler);
    bus.send(3_u32);
    bus.step(&mut world);

    let t = std::thread::spawn(move || {
        println!("Thread");
        g_bus.step((&mut graphics, &mut world));
        g_bus.step((&mut graphics, &mut world));
        g_bus.step((&mut graphics, &mut world));
        g_bus.step((&mut graphics, &mut world));
    });

    bus.send(3_u32);
    // bus.step(&mut world);
    bus.send(7_u32);
    // bus.step(&mut world);
    // bus.step(&mut world);
    // bus.step(&mut world);
    // bus.step(&mut world);
    t.join();
}
