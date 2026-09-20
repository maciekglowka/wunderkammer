use wunderkammer::Entity;

#[wunderkammer::storage]
type World = (String, u32, i32, u8);

#[test]
fn get_single() {
    let mut world = World::default();

    let entities = (0..5)
        .map(|i| {
            let e = world.spawn();
            world.insert::<u32>(e, i);
            world.insert::<i32>(e, -(i as i32));
            world.insert(e, format!("{i}"));
            e
        })
        .collect::<Vec<_>>();

    assert_eq!(2, *world.get::<&u32>(&entities[2]).unwrap());
    assert_eq!(-3, *world.get::<&i32>(&entities[3]).unwrap());
    assert_eq!("0", world.get::<&String>(&entities[0]).unwrap());
}

#[test]
fn get_single_mut() {
    let mut world = World::default();

    let entities = (0..5)
        .map(|i| {
            let e = world.spawn();
            world.insert::<u32>(e, i);
            world.insert::<i32>(e, -(i as i32));
            world.insert(e, format!("{i}"));
            e
        })
        .collect::<Vec<_>>();

    *world.get_mut::<&mut u32>(&entities[2]).unwrap() *= 7;
    assert_eq!(14, *world.get_mut::<&u32>(&entities[2]).unwrap());
}

#[test]
fn get_many() {
    let mut world = World::default();

    let entities = (0..5)
        .map(|i| {
            let e = world.spawn();
            world.insert::<u32>(e, 2 * i);
            world.insert::<u8>(e, i as u8);
            world.insert::<i32>(e, -(i as i32));
            world.insert(e, format!("{i}"));
            e
        })
        .collect::<Vec<_>>();

    assert_eq!((&4, &-2), world.get::<(&u32, &i32)>(&entities[2]).unwrap());
    assert_eq!(
        (&10, &-5, &5),
        world.get::<(&u32, &i32, &u8)>(&entities[4]).unwrap()
    );
}
