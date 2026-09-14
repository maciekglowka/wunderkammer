fn main() {
    let mut world = wunderkammer::storage_next::world::World::new();

    let a = world.spawn();
    let b = world.spawn();
    let c = world.spawn();

    world.insert(a, 1_u32);
    world.insert(b, 2_u32);

    world.insert(a, 1_i32);
    world.insert(b, 2_i32);

    for (p, v) in world.query_mut::<(u32, i32)>() {
        *p += 1;
        *v += 3;
    }
    for (p, v) in world.query::<(u32, i32)>() {
        println!("{p} {v}");
    }

    // let _ = world.get::<u32>(&entity);
    // let _ = world.get::<(u32, i32)>(&entity);

    // let mut q = world.query::<u32>();
    // while let Some(i) = world.query::<u32>().next() {
    //     //
    // }
    // let mut q = world.query::<(u32, i32)>();
    // let a = q.next();

    // world.query2::<u32, i32>();
    // world.query2::<u32, u32>();
    // let q = world.query::<(u32, i32)>();
}
