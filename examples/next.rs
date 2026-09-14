fn main() {
    let mut world = wunderkammer::storage_next::world::World::new();

    let mut q = world.query::<u32>();
    let a = q.next();
    let mut q = world.query::<(u32, i32)>();
    let a = q.next();

    // world.query2::<u32, i32>();
    // world.query2::<u32, u32>();
    // let q = world.query::<(u32, i32)>();
}
