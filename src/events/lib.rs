use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
};

struct World;
struct Graphics;

fn main() {
    let mut world = World;
    let mut graphics = Graphics;
    let mut bus = EventBus::<World>::new();
    let mut g_bus = bus.subscribe();

    let handler = EventHandler(Box::new(|ev: &u32, w: &mut World| println!("W: {ev}")));
    let g_handler = EventHandler(Box::new(|ev: &u32, w: &mut Graphics| println!("G: {ev}")));

    bus.add_handler(handler);
    g_bus.add_handler(g_handler);

    bus.send(3_u32);
    bus.step(&mut world);
    bus.send(7_u32);
    g_bus.step(&mut graphics);
    g_bus.step(&mut graphics);
    g_bus.step(&mut graphics);
    bus.step(&mut world);
    bus.step(&mut world);
    bus.step(&mut world);
    g_bus.step(&mut graphics);
    g_bus.step(&mut graphics);
}

struct ScheduledEvent(TypeId, Box<dyn Any>);

struct EventBus<W> {
    queue: Arc<Mutex<EventQueue>>,
    handlers: HashMap<TypeId, Box<dyn HandlerSetErased<W>>>,
    front: Arc<AtomicUsize>,
}
impl<W: 'static> EventBus<W> {
    pub fn new() -> Self {
        let mut queue = EventQueue::new();
        let front = queue.subscribe();
        Self {
            queue: Arc::new(Mutex::new(queue)),
            handlers: HashMap::new(),
            front,
        }
    }
    pub fn subscribe<U: 'static>(&self) -> EventBus<U> {
        let front = self.queue.lock().unwrap().subscribe();
        EventBus {
            queue: self.queue.clone(),
            handlers: HashMap::new(),
            front,
        }
    }
    pub fn add_handler<T: 'static>(&mut self, handler: EventHandler<T, W>) {
        self.handlers
            .entry(TypeId::of::<T>())
            .or_insert(Box::new(HandlerSet::<T, W>::new()))
            .add_handler(Box::new(handler));
    }
    pub fn send<T: 'static>(&self, event: T) {
        self.queue
            .lock()
            .unwrap()
            .inner
            .push_back(ScheduledEvent(TypeId::of::<T>(), Box::new(event)));
    }
    pub fn step(&mut self, world: &mut W) {
        let mut queue = self.queue.lock().unwrap();

        let Some(front) = queue.inner.get(self.front.load(Ordering::Relaxed)) else {
            return;
        };
        self.front.fetch_add(1, Ordering::Relaxed);

        if let Some(handlers) = self.handlers.get_mut(&front.0) {
            handlers.handle(&*front.1, world);
        };

        queue.synchronize();
    }
}

struct EventQueue {
    inner: VecDeque<ScheduledEvent>,
    subscriber_fronts: Vec<Weak<AtomicUsize>>,
}
impl EventQueue {
    fn new() -> Self {
        Self {
            inner: VecDeque::new(),
            subscriber_fronts: Vec::new(),
        }
    }
    fn subscribe(&mut self) -> Arc<AtomicUsize> {
        let front = Arc::new(AtomicUsize::new(self.inner.len()));
        self.subscriber_fronts.push(Arc::downgrade(&front));
        front
    }
    fn synchronize(&mut self) {
        if self.subscriber_fronts.is_empty() {
            return;
        }
        let min = self.subscriber_fronts.iter().fold(usize::MAX, |acc, s| {
            s.upgrade()
                .map(|f| f.load(Ordering::Relaxed))
                .unwrap_or(usize::MAX)
                .min(acc)
        });
        if min == 0 {
            return;
        }
        for i in (0..self.subscriber_fronts.len()).rev() {
            if let Some(f) = self.subscriber_fronts[i].upgrade() {
                f.fetch_sub(min, Ordering::Relaxed);
            } else {
                self.subscriber_fronts.remove(i);
            }
        }
        self.inner.drain(..min);
    }
}

trait HandlerSetErased<W> {
    fn add_handler(&mut self, handler: Box<dyn Any>);
    fn handle(&self, event: &dyn Any, world: &mut W);
}

struct HandlerSet<T, W> {
    handlers: Vec<EventHandler<T, W>>,
}
impl<T, W> HandlerSet<T, W> {
    fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}
impl<T: 'static, W: 'static> HandlerSetErased<W> for HandlerSet<T, W> {
    fn add_handler(&mut self, handler: Box<dyn Any>) {
        let handler = *handler.downcast().unwrap();
        self.handlers.push(handler);
    }
    fn handle(&self, event: &dyn Any, world: &mut W) {
        let event = event.downcast_ref().unwrap();
        self.handlers.iter().for_each(|h| h.execute(event, world));
    }
}

struct EventHandler<T, W>(Box<dyn Fn(&T, &mut W)>);
impl<T, W> EventHandler<T, W> {
    fn execute(&self, event: &T, world: &mut W) {
        self.0(event, world);
    }
}
