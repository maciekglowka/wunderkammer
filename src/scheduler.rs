use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
    marker::PhantomData,
};

pub struct Scheduler<W> {
    handlers: HashMap<TypeId, HandlerSet<W>>,
    queue: VecDeque<Vec<ScheduledEvent>>,
    sender: Sender,
}
impl<W: 'static> Scheduler<W> {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            queue: VecDeque::new(),
            sender: Sender::new(),
        }
    }
    pub fn subscribe<T: 'static, M>(&mut self, handler: impl IntoHandler<T, W, M>) {
        self.handlers
            .entry(TypeId::of::<T>())
            .or_insert(HandlerSet(Vec::new()))
            .subscribe(handler.handler());
    }
    pub fn send<T: 'static>(&mut self, mut event: T) {
        self.queue
            .push_back(vec![ScheduledEvent(TypeId::of::<T>(), Box::new(event))]);
    }
    pub fn step(&mut self, world: &mut W) {
        if let Some(epoch) = self.queue.pop_front() {
            for mut event in epoch {
                if let Some(set) = self.handlers.get(&event.0) {
                    set.handle(&mut event.1, world, &mut self.sender);
                }
            }
        }
    }
}

struct ScheduledEvent(TypeId, Box<dyn Any>);

pub struct Sender(Vec<ScheduledEvent>);
impl Sender {
    fn new() -> Self {
        Self(Vec::new())
    }
    pub fn send<T: 'static>(&mut self, event: T) {
        self.0
            .push(ScheduledEvent(TypeId::of::<T>(), Box::new(event)));
    }
}

pub trait IntoHandler<T, W, M> {
    fn handler(self) -> Box<dyn EventHandler<W>>;
}

impl<F, T, W> IntoHandler<T, W, EventOnlyHandler<F, T>> for F
where
    F: Fn(&mut T) + 'static,
    T: 'static,
{
    fn handler(self) -> Box<dyn EventHandler<W>> {
        Box::new(EventOnlyHandler(self, PhantomData::<T>))
    }
}

impl<F, T, W> IntoHandler<T, W, WithWorldHandler<F, T>> for F
where
    F: Fn(&mut T, &mut W) + 'static,
    T: 'static,
{
    fn handler(self) -> Box<dyn EventHandler<W>> {
        Box::new(WithWorldHandler(self, PhantomData::<T>))
    }
}

impl<F, T, W> IntoHandler<T, W, WithSenderHandler<F, T>> for F
where
    F: Fn(&mut T, &mut Sender) + 'static,
    T: 'static,
{
    fn handler(self) -> Box<dyn EventHandler<W>> {
        Box::new(WithSenderHandler(self, PhantomData::<T>))
    }
}

impl<F, T, W> IntoHandler<T, W, WithSenderAndWorldHandler<F, T>> for F
where
    F: Fn(&mut T, &mut W, &mut Sender) + 'static,
    T: 'static,
{
    fn handler(self) -> Box<dyn EventHandler<W>> {
        Box::new(WithSenderAndWorldHandler(self, PhantomData::<T>))
    }
}

pub trait EventHandler<W> {
    fn execute(&self, event: &mut Box<dyn Any>, world: &mut W, sender: &mut Sender);
}

struct EventOnlyHandler<F, T>(F, PhantomData<T>);
impl<F, T, W> EventHandler<W> for EventOnlyHandler<F, T>
where
    F: Fn(&mut T),
    T: 'static,
{
    fn execute(&self, event: &mut Box<dyn Any>, world: &mut W, _: &mut Sender) {
        let ev = event.downcast_mut().unwrap();
        self.0(ev);
    }
}

struct WithWorldHandler<F, T>(F, PhantomData<T>);
impl<F, T, W> EventHandler<W> for WithWorldHandler<F, T>
where
    F: Fn(&mut T, &mut W),
    T: 'static,
{
    fn execute(&self, event: &mut Box<dyn Any>, world: &mut W, _: &mut Sender) {
        let ev = event.downcast_mut().unwrap();
        self.0(ev, world);
    }
}

struct WithSenderHandler<F, T>(F, PhantomData<T>);
impl<F, T, W> EventHandler<W> for WithSenderHandler<F, T>
where
    F: Fn(&mut T, &mut Sender),
    T: 'static,
{
    fn execute(&self, event: &mut Box<dyn Any>, _: &mut W, sender: &mut Sender) {
        let ev = event.downcast_mut().unwrap();
        self.0(ev, sender);
    }
}

struct WithSenderAndWorldHandler<F, T>(F, PhantomData<T>);
impl<F, T, W> EventHandler<W> for WithSenderAndWorldHandler<F, T>
where
    F: Fn(&mut T, &mut W, &mut Sender),
    T: 'static,
{
    fn execute(&self, event: &mut Box<dyn Any>, world: &mut W, sender: &mut Sender) {
        let ev = event.downcast_mut().unwrap();
        self.0(ev, world, sender);
    }
}

struct HandlerSet<W>(Vec<HandlerEntry<W>>);
impl<W> HandlerSet<W> {
    fn subscribe(&mut self, handler: Box<dyn EventHandler<W>>) {
        self.0.push(HandlerEntry {
            priority: 0,
            handler,
        });
        self.0.sort_by_key(|a| a.priority);
    }
    fn handle(&self, event: &mut Box<dyn Any>, world: &mut W, sender: &mut Sender) {
        for entry in self.0.iter() {
            entry.handler.execute(event, world, sender);
        }
    }
}

struct HandlerEntry<W> {
    priority: i32,
    handler: Box<dyn EventHandler<W>>,
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_event_only() {
        // Event.
        struct Attack(u32);
        struct World;

        fn attack_handler(attack: &mut Attack) {
            attack.0 += 1;
        }

        let mut scheduler = Scheduler::new();
        scheduler.subscribe(attack_handler);

        let mut world = World;
        scheduler.send(Attack(13));
        scheduler.step(&mut world);
        // No side effects to test
    }

    #[test]
    fn test_handler_with_world() {
        // Event.
        struct Attack(u32);
        struct World(u32);

        fn attack_handler(attack: &mut Attack, world: &mut World) {
            world.0 = attack.0;
        }

        let mut scheduler = Scheduler::new();
        scheduler.subscribe(attack_handler);

        let mut world = World(0);
        scheduler.send(Attack(13));
        scheduler.step(&mut world);
        assert_eq!(13, world.0);
    }
    #[test]
    fn test_handler_with_sender() {
        // Event.
        struct Attack(u32);
        struct World;

        fn attack_handler(attack: &mut Attack, sender: &mut Sender) {
            sender.send(Attack(17 + attack.0));
        }

        let mut scheduler = Scheduler::new();
        scheduler.subscribe(attack_handler);

        let mut world = World;
        scheduler.send(Attack(13));
        scheduler.step(&mut world);
        assert_eq!(
            scheduler
                .sender
                .0
                .get(0)
                .unwrap()
                .1
                .downcast_ref::<Attack>()
                .unwrap()
                .0,
            30
        );
    }
    #[test]
    fn test_handler_with_world_and_sender() {
        // Event.
        struct Attack(u32);
        struct World(u32);

        fn attack_handler(attack: &mut Attack, world: &mut World, sender: &mut Sender) {
            world.0 = attack.0;
            sender.send(Attack(17 + attack.0));
        }

        let mut scheduler = Scheduler::new();
        scheduler.subscribe(attack_handler);

        let mut world = World(0);
        scheduler.send(Attack(13));
        scheduler.step(&mut world);
        assert_eq!(
            scheduler
                .sender
                .0
                .get(0)
                .unwrap()
                .1
                .downcast_ref::<Attack>()
                .unwrap()
                .0,
            30
        );
        assert_eq!(13, world.0)
    }
}
