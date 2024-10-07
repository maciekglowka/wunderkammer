#[cfg(test)]
mod test_derive {
    use crate::prelude::*;

    #[test]
    fn derive() {
        #[derive(Components, Default)]
        struct C {
            pub health: ComponentStorage<u32>,
            pub name: ComponentStorage<String>,
        }
        let mut c = C::default();
        let entity = Entity::default();

        c.health.insert(entity, 17);
        c.name.insert(entity, "Seventeen".to_string());

        assert_eq!(c.health.entities().len(), 1);
        assert_eq!(c.name.entities().len(), 1);

        c.despawn(entity);
        assert_eq!(c.health.entities().len(), 0);
        assert_eq!(c.name.entities().len(), 0);
    }
}
