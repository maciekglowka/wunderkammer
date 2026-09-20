use wunderkammer::{prelude::storage, storage_next::entity::Entity};

#[storage]
type Storage = (u32, i32, bool);

fn main() {
    let mut world = Storage::new();

    let a = world.spawn();
    let b = world.spawn();
    let c = world.spawn();

    world.insert(a, 1_u32);
    world.insert(b, 2_u32);

    world.insert(a, 1_i32);
    world.insert(b, 2_i32);

    for (p, v) in world.query_mut::<(&mut u32, &mut i32)>() {
        *p += 1;
        *v += 3;
    }
    for (p, v) in world.query::<(&u32, &i32)>() {
        println!("{p} {v}");
    }

    println!("{:?}", world.get::<&u32>(&a));

    let (u, i) = world.get_mut::<(&mut u32, &i32)>(&b).unwrap();

    *u += 3;
    println!("{:?}", world.get::<(&u32, &i32)>(&b));
    // let _ = world.get::<(&u32, (&i32, &u32))>(&b);
    for (e, v) in world.query::<(Entity, &i32)>() {
        println!("{e:?} {v:?}");
    }
}
