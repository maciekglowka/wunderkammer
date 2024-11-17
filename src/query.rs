/// Base query that extracts matching entities from the World struct.
#[macro_export]
macro_rules! query {
    ($world:expr, With($($component:ident),+)) => {
        {
            let with = [
                $($world.components.$component.entities()), +
            ];
            // won't fail as we match at least one component
            with.into_iter()
                .reduce(|acc, h| acc.intersection(&h).copied().collect())
                .expect("This iterator shoud never be empty!")
        }
    };
    ($world:expr, Without($($without:ident)+), With($($component:ident),+)) => {
        {
            let with = [
                $($world.components.$component.entities()), +
            ];
            let without = [
                $($world.components.$without.entities()), +
            ];
            // won't fail as we match at least one component
            let entities = with.into_iter()
                .reduce(|acc, h| acc.intersection(&h).copied().collect())
                .expect("This iterator shoud never be empty!");
            without.into_iter()
                .fold(entities, |acc, h| acc.difference(&h).copied().collect())
        }
    };
}

/// Helper query that allows to execute a given closure on each matching entity and it's components.
#[macro_export]
macro_rules! query_execute {
    ($world:expr, $(Without($($without:ident),+),)? With($($component:ident),+), $f:expr) => {{
        let entities = query!($world, $(Without($($without),+),)? With($($component),+));
        // after querying should be always safe to unwrap
        entities.iter()
            .for_each(|e| $f( *e, $($world.components.$component.get(*e).unwrap()),+ ))
    }};
}

/// Helper query that allows to execute a mutating closure on each matching entity and it's components.
#[macro_export]
macro_rules! query_execute_mut {
    ($world:expr, $(Without($($without:ident),+),)? With($($component:ident),+), $f:expr) => {{
        let entities = query!($world, $(Without($($without),+),)? With($($component),+));
        // after querying should be always safe to unwrap
        entities.iter()
            .for_each(|e| $f( *e, $($world.components.$component.get_mut(*e).unwrap()),+ ))
    }};
}

/// Query returning and immutable iterator over matching entities with their components.
#[macro_export]
macro_rules! query_iter {
    ($world:expr, $(Without($($without:ident),+),)? With($($component:ident),+)) => {
        query!($world, $(Without($($without),+),)? With($($component),+))
            .iter()
            // after querying should be always safe to unwrap
            .map(|e| ( *e, $($world.components.$component.get(*e).unwrap()),+ ))
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn query_single() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let entity = w.spawn();

        w.components.health.insert(entity, 15);
        w.components.name.insert(entity, "Fifteen".to_string());

        let entities = query!(w, With(health, name));
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&entity));
    }

    #[test]
    fn query_many() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 16);

        w.components.health.insert(c, 17);
        w.components.name.insert(c, "Seventeen".to_string());

        let entities = query!(w, With(health, name));
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&a));
        assert!(entities.contains(&c));
    }

    #[test]
    fn query_without() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 16);

        w.components.health.insert(c, 17);
        w.components.name.insert(c, "Seventeen".to_string());

        let entities = query!(w, Without(name), With(health));
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&b));
    }

    #[test]
    fn query_execute() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 16);

        w.components.health.insert(c, 17);
        w.components.name.insert(c, "Seventeen".to_string());

        let mut v = Vec::new();
        query_execute!(w, With(health, name), |_, h, n| { v.push((h, n)) });
        assert_eq!(v.len(), 2);

        // the order is not deterministic due to HashSet usage
        v.sort();
        assert_eq!(*v[0].0, 15);
        assert_eq!(v[0].1, "Fifteen");
        assert_eq!(*v[1].0, 17);
        assert_eq!(v[1].1, "Seventeen");
    }

    #[test]
    fn query_execute_without() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 16);

        w.components.health.insert(c, 17);
        w.components.name.insert(c, "Seventeen".to_string());

        let mut v = Vec::new();
        query_execute!(w, Without(name), With(health), |_, h| { v.push(h) });
        assert_eq!(v.len(), 1);
        assert_eq!(*v[0], 16);
    }

    #[test]
    fn query_execute_mut() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();

        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 17);
        w.components.name.insert(b, "Seventeen".to_string());

        query_execute_mut!(w, With(health, name), |_, h: &mut u32, n: &mut String| {
            *h += 1;
            n.insert(0, '@');
        });

        assert_eq!(*w.components.health.get(a).unwrap(), 16);
        assert_eq!(w.components.name.get(a).unwrap(), "@Fifteen");
        assert_eq!(*w.components.health.get(b).unwrap(), 18);
        assert_eq!(w.components.name.get(b).unwrap(), "@Seventeen");
    }

    #[test]
    fn query_execute_mut_without() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();

        w.components.health.insert(a, 15);

        w.components.health.insert(b, 17);
        w.components.name.insert(b, "Seventeen".to_string());

        query_execute_mut!(w, Without(name), With(health), |_, h: &mut u32| {
            *h += 1;
        });

        assert_eq!(*w.components.health.get(a).unwrap(), 16);
        assert_eq!(*w.components.health.get(b).unwrap(), 17);
    }

    #[test]
    fn query_iter() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub strength: ComponentStorage<u32>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        w.components.health.insert(a, 15);
        w.components.health.insert(c, 17);

        w.components.strength.insert(a, 1);
        w.components.strength.insert(b, 1);
        w.components.strength.insert(c, 1);

        let v = query_iter!(w, With(health, strength))
            .map(|(_, h, s)| h + s)
            .collect::<Vec<_>>();
        assert_eq!(v.len(), 2);
        assert_eq!(v.iter().sum::<u32>(), 34);
    }

    #[test]
    fn query_iter_without() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub strength: ComponentStorage<u32>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        w.components.health.insert(a, 15);
        w.components.health.insert(c, 17);

        w.components.strength.insert(a, 1);
        w.components.strength.insert(b, 2);
        w.components.strength.insert(c, 1);

        let v = query_iter!(w, Without(health), With(strength))
            .map(|(_, s)| s)
            .collect::<Vec<_>>();
        assert_eq!(v.len(), 1);
        assert_eq!(*v[0], 2);
    }

    #[test]
    fn example() {
        #[derive(ComponentSet, Default)]
        struct Components {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
            pub player: ComponentStorage<()>, // marker component
            pub poison: ComponentStorage<()>,
            pub strength: ComponentStorage<u32>,
        }

        #[derive(Default)]
        struct Resources {
            current_level: u32,
        }

        type World = WorldStorage<Components, Resources>;

        let mut world = World::default();

        let player = world.spawn();
        world.components.health.insert(player, 5);
        world.components.name.insert(player, "Player".to_string());
        world.components.player.insert(player, ());
        world.components.poison.insert(player, ());
        world.components.strength.insert(player, 3);

        let rat = world.spawn();
        world.components.health.insert(rat, 2);
        world.components.name.insert(rat, "Rat".to_string());
        world.components.strength.insert(rat, 1);

        let serpent = world.spawn();
        world.components.health.insert(serpent, 3);
        world.components.name.insert(serpent, "Serpent".to_string());
        world.components.poison.insert(serpent, ());
        world.components.strength.insert(serpent, 2);

        // find matching entities, returns HashSet<Entity>
        let npcs = query!(world, Without(player), With(health));
        assert_eq!(npcs.len(), 2);

        // apply poison
        query_execute_mut!(world, With(health, poison), |_, h: &mut u32, _| {
            *h = h.saturating_sub(1);
        });

        assert_eq!(world.components.health.get(player), Some(&4));
        assert_eq!(world.components.health.get(rat), Some(&2));
        assert_eq!(world.components.health.get(serpent), Some(&2));

        // heal player
        let _ = world.components.poison.remove(player);
        let poisoned = query!(world, With(poison));
        assert_eq!(poisoned.len(), 1);

        // use resource
        world.resources.current_level += 1;
    }
}
