/// Base query that extracts matching entities from the World struct.
#[macro_export]
macro_rules! query {
    ($world:expr, With($($components:ident), +), Without($($without:ident),+)) => {
        query!($world, With($($components),+))
            $(.filter(|&e| $world.components.$without.get(e).is_none()))+
    };
    ($world:expr, With($component:ident)) => {
        $world.components.$component.entities()
    };
    ($world:expr, With($component:ident, $($components:ident),+)) => {{
        query!($world, With($($components),+))
            .filter(|&e| $world.components.$component.get(e).is_some())
    }};
}

/// Query returning an immutable iterator over matching entities with their
/// components.
#[macro_export]
macro_rules! query_iter {
    ($world:expr, With($($components:ident), +), Without($($without:ident),+)) => {
        query_iter!($world, With($($components),+))
            $(.filter(|a| $world.components.$without.get(&a.0).is_none()))+
    };
    ($world:expr, With($component:ident)) => {
        $world
            .components
            .$component
            .entities()
            .map(|&e| (e, $world.components.$component.get(&e).unwrap()))
    };
    ($world:expr, With($component:ident, $($components:ident),+)) => {{
        query_iter!($world, With($component))
            .filter_map(|(e, c)| Some(
                (
                    e,
                    c,
                    $( $world.components.$components.get(&e)?, )+
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
            .for_each(|e| $f( e, $($world.components.$components.get_mut(&e).unwrap()),+ ))
    };
    ($world:expr, With($($components:ident), +),  $f:expr) => {
        query!($world, With($($components),+))
        // after querying should be always safe to unwrap
            .copied()
            .collect::<Vec<_>>()
            .iter()
            .for_each(|e| $f( e, $($world.components.$components.get_mut(&e).unwrap()),+ ))
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

        w.components.health.insert(entity, 15);
        w.components.name.insert(entity, "Fifteen".to_string());

        let entities = query!(w, With(health, name)).copied().collect::<Vec<_>>();
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

        let entities = query!(w, With(health, name)).copied().collect::<Vec<_>>();
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
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let a = w.spawn();
        let b = w.spawn();
        let c = w.spawn();
        let d = w.spawn();

        w.components.attack.insert(a, 2);
        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 16);
        w.components.name.insert(b, "Sixteen".to_string());

        w.components.attack.insert(c, 3);
        w.components.name.insert(c, "Seventeen".to_string());

        w.components.name.insert(d, "Eighteen".to_string());

        let entities = query!(w, With(name), Without(attack, health))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&d));

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

        w.components.health.insert(entity, 15);
        w.components.health.insert(entity_keep, 25);
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

        w.components.health.insert(entity, 15);
        w.components.health.insert(entity_keep, 25);
        w.despawn(entity);
        let entity_recycle = w.spawn();
        w.components.health.insert(entity_recycle, 35);

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

        w.components.health.insert(a, 15);
        w.components.health.insert(c, 17);

        w.components.strength.insert(a, 1);
        w.components.strength.insert(b, 1);
        w.components.strength.insert(c, 1);

        w.components.marker.insert(a, ());

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

        w.components.health.insert(a, 15);
        w.components.health.insert(c, 17);

        w.components.strength.insert(a, 1);
        w.components.strength.insert(b, 2);
        w.components.strength.insert(c, 1);

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

        w.components.player.insert(a, ());
        w.components.health.insert(c, 17);

        w.components.strength.insert(a, 1);
        w.components.strength.insert(b, 2);
        w.components.strength.insert(c, 1);

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

        w.components.health.insert(a, 15);
        w.components.name.insert(a, "Fifteen".to_string());

        w.components.health.insert(b, 17);
        w.components.name.insert(b, "Seventeen".to_string());

        query_execute!(w, With(health, name), |_, h: &mut u32, n: &mut String| {
            *h += 1;
            n.insert(0, '@');
        });

        assert_eq!(*w.components.health.get(&a).unwrap(), 16);
        assert_eq!(w.components.name.get(&a).unwrap(), "@Fifteen");
        assert_eq!(*w.components.health.get(&b).unwrap(), 18);
        assert_eq!(w.components.name.get(&b).unwrap(), "@Seventeen");
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

        w.components.health.insert(a, 15);

        w.components.health.insert(b, 17);
        w.components.name.insert(b, "Seventeen".to_string());

        query_execute!(w, With(health), Without(name), |_, h: &mut u32| {
            *h += 1;
        });

        assert_eq!(*w.components.health.get(&a).unwrap(), 16);
        assert_eq!(*w.components.health.get(&b).unwrap(), 17);
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
        let npcs = query!(world, With(health), Without(player)).collect::<Vec<_>>();
        assert_eq!(npcs.len(), 2);

        // apply poison
        query_execute!(world, With(health, poison), |_, h: &mut u32, _| {
            *h = h.saturating_sub(1);
        });

        assert_eq!(world.components.health.get(&player), Some(&4));
        assert_eq!(world.components.health.get(&rat), Some(&2));
        assert_eq!(world.components.health.get(&serpent), Some(&2));

        // heal player
        let _ = world.components.poison.remove(player);
        let poisoned = query!(world, With(poison)).collect::<Vec<_>>();
        assert_eq!(poisoned.len(), 1);

        // use resource
        world.resources.current_level += 1;
    }
}
