use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
};

use crate::observer::{ObservableQueue, Observer};

pub struct Scheduler<W> {
    handlers: HashMap<TypeId, Box<dyn HandlerSetErased<W>>>,
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
    pub fn add_system<T: 'static, M>(&mut self, handler: impl IntoHandler<T, W, M>) {
        self.add_system_with_priority::<T, M>(handler, 0);
    }
    pub fn add_system_with_priority<T: 'static, M>(
        &mut self,
        handler: impl IntoHandler<T, W, M>,
        priority: i32,
    ) {
        self.handlers
            .entry(TypeId::of::<T>())
            .or_insert(Box::new(HandlerSet::<T, W>::new()))
            .add_handler(Box::new(handler.handler()), priority);
    }
    pub fn send<T: 'static>(&mut self, event: T) {
        self.queue
            .push_back(vec![ScheduledEvent(TypeId::of::<T>(), Box::new(event))]);
    }
    pub fn step(&mut self, world: &mut W) {
        if let Some(epoch) = self.queue.pop_front() {
            for event in epoch {
                if let Some(set) = self.handlers.get_mut(&event.0) {
                    set.handle(event.1, world, &mut self.sender);
                }
            }
        }
        if !self.sender.0.is_empty() {
            self.queue.push_back(self.sender.0.drain(..).collect());
        }
    }
    pub fn observe<T: 'static>(&mut self) -> Option<Observer<T>> {
        let observer = self
            .handlers
            .entry(TypeId::of::<T>())
            .or_insert(Box::new(HandlerSet::<T, W>::new()))
            .observe();
        let boxed: Box<Observer<T>> = observer.downcast().ok()?;
        Some(*boxed)
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

pub struct EventHandler<T, W>(Box<dyn Fn(&mut T, &mut W, &mut Sender)>);

impl<T, W> EventHandler<T, W> {
    fn execute(&self, event: &mut T, world: &mut W, sender: &mut Sender) {
        self.0(event, world, sender)
    }
}

pub trait IntoHandler<T, W, M> {
    fn handler(self) -> EventHandler<T, W>;
}

impl<F, T, W> IntoHandler<T, W, EventOnlyMarker> for F
where
    F: Fn(&mut T) + 'static,
    T: 'static,
{
    fn handler(self) -> EventHandler<T, W> {
        let wrapper = move |a: &mut T, _: &mut W, _: &mut Sender| self(a);
        EventHandler::<T, W>(Box::new(wrapper))
    }
}

impl<F, T, W> IntoHandler<T, W, WithWorldMarker> for F
where
    F: Fn(&mut T, &mut W) + 'static,
    T: 'static,
{
    fn handler(self) -> EventHandler<T, W> {
        let wrapper = move |a: &mut T, w: &mut W, _: &mut Sender| self(a, w);
        EventHandler::<T, W>(Box::new(wrapper))
    }
}

impl<F, T, W> IntoHandler<T, W, WithSenderMarker> for F
where
    F: Fn(&mut T, &mut Sender) + 'static,
    T: 'static,
{
    fn handler(self) -> EventHandler<T, W> {
        let wrapper = move |a: &mut T, _: &mut W, s: &mut Sender| self(a, s);
        EventHandler::<T, W>(Box::new(wrapper))
    }
}

impl<F, T, W> IntoHandler<T, W, WithWorldAndSenderMarker> for F
where
    F: Fn(&mut T, &mut W, &mut Sender) + 'static,
    T: 'static,
{
    fn handler(self) -> EventHandler<T, W> {
        let wrapper = move |a: &mut T, w: &mut W, s: &mut Sender| self(a, w, s);
        EventHandler::<T, W>(Box::new(wrapper))
    }
}

// Markers
struct EventOnlyMarker;
struct WithWorldMarker;
struct WithSenderMarker;
struct WithWorldAndSenderMarker;

trait HandlerSetErased<W> {
    fn add_handler(&mut self, handler: Box<dyn Any>, priority: i32);
    fn handle(&mut self, event: Box<dyn Any>, world: &mut W, sender: &mut Sender);
    fn observe(&mut self) -> Box<dyn Any>;
}

struct HandlerSet<T, W> {
    handlers: Vec<HandlerEntry<T, W>>,
    observable: ObservableQueue<T>,
}
impl<T, W> HandlerSet<T, W> {
    fn new() -> Self {
        Self {
            handlers: Vec::new(),
            observable: ObservableQueue::new(),
        }
    }
}
impl<T: 'static, W: 'static> HandlerSetErased<W> for HandlerSet<T, W> {
    fn add_handler(&mut self, handler: Box<dyn Any>, priority: i32) {
        let h = *handler.downcast().unwrap();
        self.handlers.push(HandlerEntry {
            priority,
            handler: h,
        });
        self.handlers.sort_by_key(|a| a.priority);
    }
    fn handle(&mut self, event: Box<dyn Any>, world: &mut W, sender: &mut Sender) {
        let mut ev = event.downcast::<T>().unwrap();
        for entry in self.handlers.iter() {
            entry.handler.execute(ev.as_mut(), world, sender);
        }
        self.observable.push(*ev);
    }
    fn observe(&mut self) -> Box<dyn Any> {
        Box::new(self.observable.subscribe())
    }
}

struct HandlerEntry<T, W> {
    priority: i32,
    handler: EventHandler<T, W>,
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
        scheduler.add_system(attack_handler);

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
        scheduler.add_system(attack_handler);

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
        scheduler.add_system(attack_handler);

        let mut world = World;
        scheduler.send(Attack(13));
        scheduler.step(&mut world);
        assert_eq!(
            scheduler
                .queue
                .get(0)
                .unwrap()
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
        scheduler.add_system(attack_handler);

        let mut world = World(0);
        scheduler.send(Attack(13));
        scheduler.step(&mut world);
        assert_eq!(
            scheduler
                .queue
                .get(0)
                .unwrap()
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
    #[test]
    fn test_handler_epochs() {
        // Event.
        struct Attack(u32);
        struct World(u32);

        fn attack_handler(attack: &mut Attack, world: &mut World, sender: &mut Sender) {
            world.0 += attack.0;
            sender.send(Attack(attack.0));
        }

        let mut scheduler = Scheduler::new();
        scheduler.add_system(attack_handler);

        let mut world = World(0);
        scheduler.send(Attack(1));
        for _ in 0..5 {
            scheduler.step(&mut world);
        }
        assert_eq!(5, world.0)
    }
    #[test]
    fn test_resulting_event() {
        // Events.
        struct Attack(u32);
        struct Damage(u32);

        struct World(u32);

        fn attack_handler(attack: &mut Attack, sender: &mut Sender) {
            sender.send(Damage(2 * attack.0));
        }

        fn damage_handler(damage: &mut Damage, world: &mut World) {
            world.0 = damage.0;
        }

        let mut scheduler = Scheduler::new();
        scheduler.add_system(attack_handler);
        scheduler.add_system(damage_handler);

        let mut world = World(0);
        scheduler.send(Attack(3));

        for _ in 0..2 {
            scheduler.step(&mut world);
        }
        assert_eq!(6, world.0)
    }
    #[test]
    fn test_priority() {
        // Events.
        struct Attack(u32);
        struct World(u32);

        fn add_handler(attack: &mut Attack, world: &mut World) {
            world.0 = attack.0 + 2;
        }
        fn multiply_handler(attack: &mut Attack) {
            attack.0 *= 3;
        }

        let mut scheduler = Scheduler::new();
        scheduler.add_system_with_priority(add_handler, 1);
        scheduler.add_system_with_priority(multiply_handler, 0);

        let mut world = World(0);
        scheduler.send(Attack(4));

        scheduler.step(&mut world);
        assert_eq!(4 * 3 + 2, world.0)
    }

    #[test]
    fn test_observe() {
        // Events.
        struct Attack(u32);
        struct Damage(u32);

        struct World(u32);

        fn attack_handler(attack: &mut Attack, sender: &mut Sender) {
            sender.send(Damage(2 * attack.0));
        }

        fn damage_handler(damage: &mut Damage, world: &mut World) {
            world.0 = damage.0;
        }

        let mut scheduler = Scheduler::new();
        scheduler.add_system(attack_handler);
        scheduler.add_system(damage_handler);

        let mut world = World(0);

        let damage_observer = scheduler.observe::<Damage>().unwrap();

        scheduler.send(Attack(3));

        for _ in 0..2 {
            scheduler.step(&mut world);
        }

        assert_eq!(Some(6), damage_observer.map_next(|a| a.0));
    }

    #[test]
    fn test_observe_before() {
        // Event.
        struct Attack(u32);

        struct World;

        fn attack_handler(_: &mut Attack) {
            // idle
        }

        let mut scheduler = Scheduler::new();
        let attack_observer = scheduler.observe::<Attack>().unwrap();

        scheduler.add_system(attack_handler);

        let mut world = World;

        scheduler.send(Attack(3));
        scheduler.step(&mut world);

        assert_eq!(Some(3), attack_observer.map_next(|a| a.0));
    }
}
