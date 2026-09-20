use wunderkammer::Entity;

#[wunderkammer::storage]
type World = (String, u32, i32, u8);

#[test]
fn query() {
    let mut world = World::default();

    let a = world.spawn();
    let b = world.spawn();
    let c = world.spawn();
    let d = world.spawn();

    world.insert::<u32>(a, 1);
    world.insert::<i32>(a, -2);
    world.insert::<String>(a, "1-2".to_string());

    world.insert::<u32>(b, 2);
    world.insert::<i32>(b, -4);

    world.insert::<u32>(c, 3);
    world.insert::<i32>(c, -4);
    world.insert::<String>(c, "3-4".to_string());

    world.insert::<u32>(d, 5);
    world.insert::<String>(d, "5".to_string());

    for (e, u, i, s) in world.query::<(Entity, &u32, &i32, &String)>() {
        match e {
            ee if ee == a => {
                assert_eq!(u, &1);
                assert_eq!(i, &-2);
                assert_eq!(s.as_str(), "1-2");
            }
            ee if ee == c => {
                assert_eq!(u, &3);
                assert_eq!(i, &-4);
                assert_eq!(s.as_str(), "3-4");
            }
            _ => panic!("invalid query entity"),
        };
    }
}

#[test]
fn query_mut() {
    let mut world = World::default();

    let a = world.spawn();
    let b = world.spawn();
    let c = world.spawn();
    let d = world.spawn();

    world.insert::<u32>(a, 1);
    world.insert::<i32>(a, -2);
    world.insert::<String>(a, "1-2".to_string());

    world.insert::<u32>(b, 2);
    world.insert::<i32>(b, -4);

    world.insert::<u32>(c, 3);
    world.insert::<i32>(c, -4);
    world.insert::<String>(c, "3-4".to_string());

    world.insert::<u32>(d, 5);
    world.insert::<String>(d, "5".to_string());

    for (e, u, _, _) in world.query_mut::<(Entity, &mut u32, &i32, &String)>() {
        match e {
            ee if ee == a => {
                *u *= 2;
            }
            ee if ee == c => {
                *u *= 3;
            }
            _ => panic!("invalid query entity"),
        }
    }

    assert_eq!(&2, world.get::<&u32>(&a).unwrap());
    assert_eq!(&9, world.get::<&u32>(&c).unwrap());
}

#[test]
fn query_option() {
    let mut world = World::default();

    let a = world.spawn();
    let b = world.spawn();
    let c = world.spawn();

    world.insert::<u32>(a, 1);
    world.insert::<i32>(a, -2);
    world.insert::<u8>(a, 2);

    world.insert::<u32>(b, 2);
    world.insert::<i32>(b, -4);

    world.insert::<u32>(c, 3);
    world.insert::<i32>(c, -4);
    world.insert::<u8>(c, 6);

    for (u, _, uu) in world.query::<(&u32, &i32, Option<&u8>)>() {
        match uu {
            Some(uu) => assert_eq!(*uu, 2 * *u as u8),
            None => assert_eq!(2, *u),
        }
    }
}

#[test]
fn query_option_mut() {
    let mut world = World::default();

    let a = world.spawn();
    let b = world.spawn();
    let c = world.spawn();

    world.insert::<u32>(a, 1);
    world.insert::<i32>(a, -2);
    world.insert::<u8>(a, 2);

    world.insert::<u32>(b, 2);
    world.insert::<i32>(b, -4);

    world.insert::<u32>(c, 3);
    world.insert::<i32>(c, -4);
    world.insert::<u8>(c, 6);

    for (u, _, uu) in world.query_mut::<(&mut u32, &i32, Option<&u8>)>() {
        if let Some(uu) = uu {
            *u += *uu as u32;
        }
    }

    assert_eq!(&3, world.get::<&u32>(&a).unwrap());
    assert_eq!(&2, world.get::<&u32>(&b).unwrap());
    assert_eq!(&9, world.get::<&u32>(&c).unwrap());
}

#[test]
fn query_without() {
    let mut world = World::default();

    let a = world.spawn();
    let b = world.spawn();
    let c = world.spawn();

    world.insert::<u32>(a, 1);
    world.insert::<i32>(a, -2);

    world.insert::<u32>(b, 2);
    world.insert::<i32>(b, -4);

    world.insert::<u32>(c, 3);
    world.insert::<i32>(c, -4);
    world.insert::<u8>(c, 6);

    let s: i32 = world
        .query::<(&u32, &i32)>()
        .without::<u8>()
        .map(|(u, i)| *u as i32 * i)
        .sum();

    assert_eq!(s, -2 + 2 * -4);
}

#[test]
fn query_without_mut() {
    let mut world = World::default();

    let a = world.spawn();
    let b = world.spawn();
    let c = world.spawn();

    world.insert::<u32>(a, 1);
    world.insert::<i32>(a, -2);

    world.insert::<u32>(b, 2);

    world.insert::<u32>(c, 3);
    world.insert::<i32>(c, -4);
    world.insert::<u8>(c, 6);

    for u in world
        .query_mut::<&mut u32>()
        .without::<u8>()
        .without::<i32>()
    {
        *u = 0;
    }

    assert_eq!(&1, world.get::<&u32>(&a).unwrap());
    assert_eq!(&0, world.get::<&u32>(&b).unwrap());
    assert_eq!(&3, world.get::<&u32>(&c).unwrap());
}
