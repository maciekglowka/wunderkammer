use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

pub struct Scheduler<W> {
    handlers: HashMap<TypeId, Box<dyn HandlerSetErased<W>>>,
}
impl<W: 'static> Scheduler<W> {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    pub fn subscribe<T: 'static>(&mut self, handler: impl IntoHandler<T, W>) {
        self.handlers
            .entry(TypeId::of::<T>())
            .or_insert(Box::new(HandlerSet(Vec::new())))
            .subscribe(handler.handler());
    }
    pub fn handle<T: 'static>(&self, mut event: T, world: &mut W) {
        if let Some(set) = self.handlers.get(&TypeId::of::<T>()) {
            set.handle(&mut event, world);
        }
    }
}

pub trait IntoHandler<T, W> {
    fn handler(self) -> Box<dyn EventHandler<W>>;
}
impl<F, T, W> IntoHandler<T, W> for F
where
    F: Fn(&mut T, &mut W) + 'static,
    T: 'static,
{
    fn handler(self) -> Box<dyn EventHandler<W>> {
        Box::new(FunctionHandler(self, PhantomData::<T>))
    }
}

pub trait EventHandler<W> {
    fn execute(&self, event: &mut dyn Any, world: &mut W);
}

struct FunctionHandler<F, T>(F, PhantomData<T>);
impl<F, T, W> EventHandler<W> for FunctionHandler<F, T>
where
    F: Fn(&mut T, &mut W),
    T: 'static,
{
    fn execute(&self, event: &mut dyn Any, world: &mut W) {
        let ev = event.downcast_mut().unwrap();
        self.0(ev, world);
    }
}

trait HandlerSetErased<W> {
    fn subscribe(&mut self, handler: Box<dyn EventHandler<W>>);
    fn handle(&self, event: &mut dyn Any, world: &mut W);
}

struct HandlerSet<W>(Vec<Box<dyn EventHandler<W>>>);
impl<W> HandlerSetErased<W> for HandlerSet<W> {
    fn subscribe(&mut self, handler: Box<dyn EventHandler<W>>) {
        self.0.push(handler);
    }
    fn handle(&self, event: &mut dyn Any, world: &mut W) {
        for handler in self.0.iter() {
            handler.execute(event, world);
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_single_handler() {
        // Event.
        struct Attack(u32);
        struct World(u32);

        fn attack_handler(attack: &mut Attack, world: &mut World) {
            world.0 = attack.0;
        }

        let mut scheduler = Scheduler::new();
        scheduler.subscribe(attack_handler);

        let mut world = World(0);
        scheduler.handle(Attack(13), &mut world);
        assert_eq!(13, world.0);
    }
}
