use wunderkammer::prelude::*;

use std::time::Instant;

fn main() {
    println!("Test 2");
    test_two();
    println!("Test 8");
    // test_eight();
}

// fn test_eight() {
//     #[derive(ComponentSet, Default)]
//     struct C {
//         pub c0: ComponentStorage<u32>,
//         pub c1: ComponentStorage<u32>,
//         pub c2: ComponentStorage<u32>,
//         pub c3: ComponentStorage<u32>,
//         pub c4: ComponentStorage<u32>,
//         pub c5: ComponentStorage<u32>,
//         pub c6: ComponentStorage<u32>,
//         pub c7: ComponentStorage<u32>,
//     }
//     #[derive(Default)]
//     struct R;
//     let mut w = WorldStorage::<C, R>::default();

//     let count = 10000;

//     for _ in 0..count {
//         let a = w.spawn();
//         w.components.c0.insert(a, 15);
//         w.components.c1.insert(a, 15);
//         w.components.c2.insert(a, 15);
//         w.components.c3.insert(a, 15);
//         w.components.c4.insert(a, 15);
//         w.components.c5.insert(a, 15);
//         w.components.c6.insert(a, 15);
//         w.components.c7.insert(a, 15);

//         let b = w.spawn();
//         w.components.c0.insert(a, 15);
//         w.components.c1.insert(a, 15);
//         w.components.c2.insert(a, 15);
//         w.components.c3.insert(a, 15);
//     }

//     let now = Instant::now();
//     let entities = query!(w, With(c0, c1, c2, c3, c4, c5, c6, c7));
//     println!("HashSet: {:?}", now.elapsed());
//     assert_eq!(entities.len(), count);

//     let now = Instant::now();
//     let entities = query_next!(w, With(c0, c1, c2, c3, c4, c5, c6, c7))
//         .copied()
//         .collect::<Vec<_>>();
//     println!("Lazy with collect: {:?}", now.elapsed());
//     assert_eq!(entities.len(), count);

//     let now = Instant::now();
//     for e in query_next!(w, With(c0, c1, c2, c3, c4, c5, c6, c7)) {
//         //
//         // let a = *e;
//     }
//     println!("Lazy no collect: {:?}", now.elapsed());
// }

fn test_two() {
    #[derive(ComponentSet, Default)]
    struct C {
        pub health: ComponentStorage<u32>,
        pub name: ComponentStorage<String>,
    }
    #[derive(Default)]
    struct R;
    let mut w = WorldStorage::<C, R>::default();

    let count = 10000;

    for _ in 0..count {
        let a = w.spawn();
        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        let b = w.spawn();
        w.components.health.insert(b, 15);
    }

    let now = Instant::now();
    let entities = query!(w, With(health, name)).copied().collect::<Vec<_>>();
    println!("Lazy with collect: {:?}", now.elapsed());
    assert_eq!(entities.len(), count);

    let now = Instant::now();
    for e in query!(w, With(health, name)) {
        //
        // let a = *e;
    }
    println!("Lazy no collect: {:?}", now.elapsed());
}
