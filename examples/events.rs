use wunderkammer::prelude::{event_bus, EventSubscriber};

struct World(u32);
struct Graphics(u32);

fn main() {
    let mut world = World(0);
    let mut graphics = Graphics(0);
    let mut bus = event_bus();
    // let mut g_bus = bus.spawn_subscriber();

    // let handler = EventHandler(Box::new(|ev: &u32, w: &mut World| println!("W:
    // {ev}")));
    let handler = |ev: &u32, w: &World| println!("W: {ev}");
    let g_handler = |ev: &u32, w: &mut (&mut Graphics, &World)| {
        w.0 .0 += 1;
        println!("G: {ev} {}", w.0 .0);
    };

    // let g_handler = EventHandler(Box::new(|ev: &u32, w: (&mut Graphics, &mut
    // World)| {     println!("G: {ev}")
    // }));

    bus.add_handler(handler);
    // g_bus.add_handler(g_handler);

    bus.send(3_u32);
    bus.step(&world);
    bus.send(7_u32);

    // g_bus.step(&mut (&mut graphics, &world));

    // g_bus.step(&mut (&mut graphics, &world));
    // g_bus.step(&mut graphics);
    // g_bus.step(&mut graphics);
    // bus.step(&mut world);
    // bus.step(&mut world);
    // bus.step(&mut world);
    // g_bus.step(&mut graphics);
    // g_bus.step(&mut graphics);
}
