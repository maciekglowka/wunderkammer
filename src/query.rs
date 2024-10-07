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
    ($world:expr, Without($($without:ident),+), With($($component:ident),+)) => {
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

#[macro_export]
macro_rules! query_execute {
    ($world:expr, $(Without($($without:ident),+),)? With($($component:ident),+), $f:expr) => {{
        let entities = query!($world, $(Without($($without),+))? With($($component),+));
        // after querying should be always safe to unwrap
        entities.iter()
            .for_each(|e| $f( $($world.components.$component.get(*e).unwrap()),+ ))
    }};
}

#[macro_export]
macro_rules! query_execute_mut {
    ($world:expr, $(Without($($without:ident),+),)? With($($component:ident),+), $f:expr) => {{
        let entities = query!($world, $(Without($($without),+))? With($($component),+));
        // after querying should be always safe to unwrap
        entities.iter()
            .for_each(|e| $f( $($world.components.$component.get_mut(*e).unwrap()),+ ))
    }};
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn query_single() {
        #[derive(Components, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        let mut w = WorldStorage::<C>::default();
        let entity = w.spawn();

        w.components.health.insert(entity, 15);
        w.components.name.insert(entity, "Fifteen".to_string());

        let entities = query!(w, With(health, name));
        assert_eq!(entities.len(), 1);
        assert_eq!(entities.contains(&entity), true);
    }

    #[test]
    fn query_many() {
        #[derive(Components, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        let mut w = WorldStorage::<C>::default();
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
        assert_eq!(entities.contains(&a), true);
        assert_eq!(entities.contains(&c), true);
    }

    #[test]
    fn query_without() {
        #[derive(Components, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        let mut w = WorldStorage::<C>::default();
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
        assert_eq!(entities.contains(&b), true);
    }

    #[test]
    fn query_execute() {
        #[derive(Components, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        let mut w = WorldStorage::<C>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 16);

        w.components.health.insert(c, 17);
        w.components.name.insert(c, "Seventeen".to_string());

        let mut v = Vec::new();
        query_execute!(w, With(health, name), |h, n| { v.push((h, n)) });
        assert_eq!(v.len(), 2);

        // the order is not deterministic due to HashSet usage
        v.sort();
        assert_eq!(*v[0].0, 15);
        assert_eq!(v[0].1, "Fifteen");
        assert_eq!(*v[1].0, 17);
        assert_eq!(v[1].1, "Seventeen");
    }

    #[test]
    fn query_execute_mut() {
        #[derive(Components, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        let mut w = WorldStorage::<C>::default();
        let a = w.spawn();
        let b = w.spawn();

        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 17);
        w.components.name.insert(b, "Seventeen".to_string());

        query_execute_mut!(w, With(health, name), |h: &mut u32, n: &mut String| {
            *h += 1;
            n.insert(0, '@');
        });

        assert_eq!(*w.components.health.get(a).unwrap(), 16);
        assert_eq!(w.components.name.get(a).unwrap(), "@Fifteen");
        assert_eq!(*w.components.health.get(b).unwrap(), 18);
        assert_eq!(w.components.name.get(b).unwrap(), "@Seventeen");
    }
}
