// This test is form an older version.
// Should be redone.

#[cfg(test)]
mod test_derive {
    use wunderkammer::prelude::*;

    #[test]
    fn derive() {
        #[derive(ComponentSet, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        let mut c = C::default();
        let entity = Entity::default();

        c.health.__insert(entity, 17);
        c.name.__insert(entity, "Seventeen".to_string());

        assert_eq!(c.health.entities().collect::<Vec<_>>().len(), 1);
        assert_eq!(c.name.entities().collect::<Vec<_>>().len(), 1);

        c.remove_all_components(entity);
        assert_eq!(c.health.entities().collect::<Vec<_>>().len(), 0);
        assert_eq!(c.name.entities().collect::<Vec<_>>().len(), 0);
    }
}
