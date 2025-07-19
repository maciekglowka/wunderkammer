#[macro_export]
macro_rules! insert {
    ($world:expr,  $entity:expr, $component:ident, $value:expr) => {
        if $world.is_valid($entity) {
            $world.components.$component.insert($entity, $value);
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_insert() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();
        let entity = w.spawn();

        insert!(w, health, entity, 15);
        assert_eq!(w.components.health.get(entity), Some(&15));
    }

    #[test]
    fn test_invalid() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
        }
        #[derive(Default)]
        struct R;
        let mut w = WorldStorage::<C, R>::default();

        let entity = Entity { id: 2, version: 0 };

        insert!(w, health, entity, 15);
        assert_eq!(w.components.health.get(entity), None);
    }
}
