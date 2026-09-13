use std::rc::Rc;

use wunderkammer::events::markers::Mut;
use wunderkammer::events::EventDispatcher;
use wunderkammer::prelude::{BusHandle, EventSender};

struct World(u32);
struct Graphics(u32);

fn main() {
    let mut world = World(0);
    let mut gfx = Graphics(0);
    let mut bus: BusHandle<World> = BusHandle::default();
    let mut gfx_bus: BusHandle<(Graphics, World), (Mut, Mut)> = bus.spawn_handle();

    let handler = |ev: &u32, w: &mut World| {
        w.0 += ev;
        Ok(())
    };
    let gfx_handler = |ev: &u32, (gfx, w): (&mut Graphics, &mut World)| {
        gfx.0 += 2 * w.0;
        Ok(())
    };

    let sender = |ev: &u32, w: &mut World, s: &mut EventSender| {
        s.send(Rc::new(*ev));
        Ok(())
    };
    let non_send_handler = |ev: &Rc<u32>, w: &mut World| Ok(());

    bus.add_handler(handler);
    bus.add_handler(sender);
    bus.add_handler(non_send_handler);
    gfx_bus.add_handler(gfx_handler);

    bus.send(3_u32);
    bus.step(&mut world);
    gfx_bus.step((&mut gfx, &mut world));

    assert_eq!(world.0, 3);
    assert_eq!(gfx.0, 6);
}
