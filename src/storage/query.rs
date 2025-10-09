/// Base query that extracts matching entities from the World struct.
#[macro_export]
macro_rules! query {
    ($world:expr, With($($components:ident), +), Without($($without:ident),+)) => {
        query!($world, With($($components),+))
            $(.filter(|&e| $world.cmp.$without.get(e).is_none()))+
    };
    ($world:expr, With($component:ident)) => {
        $world.cmp.$component.entities()
    };
    ($world:expr, With($component:ident, $($components:ident),+)) => {{
        query!($world, With($($components),+))
            .filter(|&e| $world.cmp.$component.get(e).is_some())
    }};
}

/// Query returning an immutable iterator over matching entities with their
/// components.
#[macro_export]
macro_rules! query_iter {
    ($world:expr, With($($components:ident), +), Without($($without:ident),+)) => {
        query_iter!($world, With($($components),+))
            $(.filter(|a| $world.cmp.$without.get(&a.0).is_none()))+
    };
    ($world:expr, With($component:ident)) => {
        $world
            .cmp
            .$component
            .entities()
            .map(|&e| (e, $world.cmp.$component.get(&e).unwrap()))
    };
    ($world:expr, With($component:ident, $($components:ident),+)) => {{
        query_iter!($world, With($component))
            .filter_map(|(e, c)| Some(
                (
                    e,
                    c,
                    $( $world.cmp.$components.get(&e)?, )+
                )
            ))
    }};
}

/// Helper query that allows to execute a mutating closure on each matching
/// entity and it's components.
#[macro_export]
macro_rules! query_execute {
    ($world:expr, With($($components:ident), +), Without($($without:ident),+), $f:expr) => {
        query!($world, With($($components),+), Without($($without),+))
        // after querying should be always safe to unwrap
            .copied()
            .collect::<Vec<_>>()
            .iter()
            .for_each(|e| $f( e, $($world.cmp.$components.get_mut(&e).unwrap()),+ ))
    };
    ($world:expr, With($($components:ident), +),  $f:expr) => {
        query!($world, With($($components),+))
        // after querying should be always safe to unwrap
            .copied()
            .collect::<Vec<_>>()
            .iter()
            .for_each(|e| $f( e, $($world.cmp.$components.get_mut(&e).unwrap()),+ ))
    };
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

        insert!(w, health, entity, 15);
        insert!(w, name, entity, "Fifteen".to_string());

        let entities = query!(w, With(health, name)).copied().collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&entity));
    }

    #[test]
    fn query_many() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub marker: ComponentStorage<()>,
            pub name: ComponentStorage<String>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        insert!(w, health, a, 15);
        insert!(w, name, a, "Fifteen".to_string());
        insert!(w, marker, a, ());

        insert!(w, health, b, 16);

        insert!(w, health, c, 17);
        insert!(w, name, c, "Seventeen".to_string());

        let entities = query!(w, With(health, name)).copied().collect::<Vec<_>>();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&a));
        assert!(entities.contains(&c));

        let entities = query!(w, With(health, name, marker))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);
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

        insert!(w, health, a, 15);
        insert!(w, name, a, "Fifteen".to_string());

        insert!(w, health, b, 16);

        insert!(w, health, c, 17);
        insert!(w, name, c, "Seventeen".to_string());

        let entities = query!(w, With(health), Without(name))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&b));
    }

    #[test]
    fn query_without_many() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub attack: ComponentStorage<u32>,
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
            pub marker: ComponentStorage<()>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();
        let d = w.spawn();

        insert!(w, attack, a, 2);
        insert!(w, health, a, 15);
        insert!(w, name, a, "Fifteen".to_string());

        insert!(w, health, b, 16);
        insert!(w, name, b, "Sixteen".to_string());

        insert!(w, attack, c, 3);
        insert!(w, name, c, "Seventeen".to_string());

        insert!(w, name, d, "Eighteen".to_string());
        insert!(w, marker, d, ());

        let entities = query!(w, With(name), Without(attack, health))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&d));

        let entities = query!(w, With(name), Without(attack, health, marker))
            .copied()
            .collect::<Vec<_>>();
        assert!(entities.is_empty());

        let entities = query!(w, With(attack, name), Without(health))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&c));
    }

    #[test]
    fn query_after_despawn() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let entity = w.spawn();
        let entity_keep = w.spawn();

        insert!(w, health, entity, 15);
        insert!(w, health, entity_keep, 25);
        w.despawn(entity);

        let entities = query!(w, With(health)).copied().collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&entity_keep));
    }

    #[test]
    fn query_after_recycle() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let entity = w.spawn();
        let entity_keep = w.spawn();

        insert!(w, health, entity, 15);
        insert!(w, health, entity_keep, 25);
        w.despawn(entity);
        let entity_recycle = w.spawn();
        insert!(w, health, entity_recycle, 35);

        assert_eq!(entity.id, entity_recycle.id);
        assert_ne!(entity.version, entity_recycle.version);

        let entities = query!(w, With(health)).copied().collect::<Vec<_>>();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&entity_keep));
        assert!(entities.contains(&entity_recycle));
    }

    #[test]
    fn query_iter() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub marker: ComponentStorage<()>,
            pub strength: ComponentStorage<u32>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        insert!(w, health, a, 15);
        insert!(w, health, c, 17);

        insert!(w, strength, a, 1);
        insert!(w, strength, b, 1);
        insert!(w, strength, c, 1);

        insert!(w, marker, a, ());

        let v = query_iter!(w, With(strength))
            .map(|(_, s)| *s)
            .collect::<Vec<_>>();

        assert_eq!(v.len(), 3);
        assert_eq!(v.iter().sum::<u32>(), 3);

        let v = query_iter!(w, With(health, strength))
            .map(|(_, h, s)| h + s)
            .collect::<Vec<_>>();

        assert_eq!(v.len(), 2);
        assert_eq!(v.iter().sum::<u32>(), 34);

        let v = query_iter!(w, With(health, strength, marker))
            .map(|(_, h, s, _)| h + s)
            .collect::<Vec<_>>();

        assert_eq!(v.len(), 1);
        assert_eq!(v.iter().sum::<u32>(), 16);
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

        insert!(w, health, a, 15);
        insert!(w, health, c, 17);

        insert!(w, strength, a, 1);
        insert!(w, strength, b, 2);
        insert!(w, strength, c, 1);

        let v = query_iter!(w, With(strength), Without(health))
            .map(|(_, s)| s)
            .collect::<Vec<_>>();

        assert_eq!(v.len(), 1);
        assert_eq!(*v[0], 2);
    }

    #[test]
    fn query_iter_without_many() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub player: ComponentStorage<()>,
            pub health: ComponentStorage<u32>,
            pub strength: ComponentStorage<u32>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();

        insert!(w, player, a, ());
        insert!(w, health, c, 17);

        insert!(w, strength, a, 1);
        insert!(w, strength, b, 2);
        insert!(w, strength, c, 1);

        let v = query_iter!(w, With(strength), Without(health, player))
            .map(|(_, s)| s)
            .collect::<Vec<_>>();
        assert_eq!(v.len(), 1);
        assert_eq!(*v[0], 2);
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

        insert!(w, health, a, 15);
        insert!(w, name, a, "Fifteen".to_string());

        insert!(w, health, b, 17);
        insert!(w, name, b, "Seventeen".to_string());

        query_execute!(w, With(health, name), |_, h: &mut u32, n: &mut String| {
            *h += 1;
            n.insert(0, '@');
        });

        assert_eq!(*w.cmp.health.get(&a).unwrap(), 16);
        assert_eq!(w.cmp.name.get(&a).unwrap(), "@Fifteen");
        assert_eq!(*w.cmp.health.get(&b).unwrap(), 18);
        assert_eq!(w.cmp.name.get(&b).unwrap(), "@Seventeen");
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

        insert!(w, health, a, 15);

        insert!(w, health, b, 17);
        insert!(w, name, b, "Seventeen".to_string());

        query_execute!(w, With(health), Without(name), |_, h: &mut u32| {
            *h += 1;
        });

        assert_eq!(*w.cmp.health.get(&a).unwrap(), 16);
        assert_eq!(*w.cmp.health.get(&b).unwrap(), 17);
    }
}
