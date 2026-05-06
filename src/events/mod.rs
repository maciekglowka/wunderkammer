use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
};

struct ScheduledEvent(TypeId, Box<dyn Any>);

pub fn event_bus<W>() -> EventSubscriber<W> {
    let mut queue = EventQueue::new();
    let front = queue.subscribe();
    EventSubscriber {
        queue: Arc::new(Mutex::new(queue)),
        handlers: HashMap::new(),
        front,
    }
}

pub struct EventSubscriber<W> {
    queue: Arc<Mutex<EventQueue>>,
    handlers: HashMap<TypeId, Vec<Box<dyn Handler<W>>>>,
    front: Arc<AtomicUsize>,
}
impl<W> EventSubscriber<W> {
    pub fn spawn_subscriber<U>(&self) -> EventSubscriber<U> {
        let front = self.queue.lock().unwrap().subscribe();
        EventSubscriber {
            queue: self.queue.clone(),
            handlers: HashMap::new(),
            front,
        }
    }
    pub fn add_handler<F, T>(&mut self, handler: F)
    where
        F: IntoWrapper<F, T, W>,
        T: 'static,
    {
        let wrapper = handler.wrap();
        self.handlers
            .entry(TypeId::of::<T>())
            .or_insert(vec![])
            .push(Box::new(wrapper));
    }
    pub fn send<T: 'static>(&self, event: T) {
        self.queue
            .lock()
            .unwrap()
            .inner
            .push_back(ScheduledEvent(TypeId::of::<T>(), Box::new(event)));
    }
    pub fn step(&mut self, env: W) {
        let mut queue = self.queue.lock().unwrap();

        let Some(front) = queue.inner.get(self.front.load(Ordering::Relaxed)) else {
            return;
        };
        self.front.fetch_add(1, Ordering::Relaxed);

        if let Some(handlers) = self.handlers.get(&front.0) {
            // let handler = handlers.first().unwrap().as_ref();
            // handler.handle(&*front.1, env);
            // env.accept(&*front.1, handlers.first().unwrap().as_ref());
            // env.accept_handlers(&*front.1, handlers.iter());
            // handlers.iter().for_each(|h| h.handle(&*front.1, env));
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

trait HandlerEnv {
    // type Target;
    // fn accept_handlers(
    //     self,
    //     arg: &dyn Any,
    //     handlers: impl IntoIterator<Item = &'a Box<dyn Handler<W>>>,
    // );
    fn accept(&mut self, arg: &dyn Any, handler: &dyn Handler<Self>);
}
impl<'a, W: 'a> HandlerEnv for &'a mut W {
    // type Target = &'a mut W;

    fn accept(&mut self, arg: &dyn Any, handler: &dyn Handler<&mut W>) {
        handler.handle(arg, *self);
    }
}
// impl<'a, W: 'a> HandlerEnv<'a, &'a mut W> for &'a mut W {
// fn accept_handlers(
//     self,
//     arg: &dyn Any,
//     handlers: impl IntoIterator<Item = &'a Box<dyn Handler<&'a mut W>>>,
// ) {
//     for handler in handlers {
//         // handler.handle(arg, self);
//         println!("Ref");
//     }
// }
// }
// impl<'a, W> HandlerEnv<'a, W> for &mut W {
//     fn accept_handlers(self, arg: &dyn Any, handlers: impl Iterator<Item =
// &'a impl Handler<W>>) {         println!("Ref mut");
//     }
// }

trait Handler<W> {
    fn handle(&self, arg: &dyn Any, env: W);
}

struct HandlerWrapper<F, T> {
    f: F,
    _marker: std::marker::PhantomData<T>,
}

impl<F, T, W> Handler<W> for HandlerWrapper<F, T>
where
    F: Fn(&T, W),
    T: 'static,
{
    fn handle(&self, arg: &dyn Any, env: W) {
        let arg = arg.downcast_ref().unwrap();
        (self.f)(arg, env);
    }
}

trait IntoWrapper<F, T, W> {
    fn wrap(self) -> HandlerWrapper<impl Fn(&T, W) + 'static, T>;
}
impl<F, T, W> IntoWrapper<F, T, W> for F
where
    F: Fn(&T, W) + 'static,
    T: 'static,
{
    fn wrap(self) -> HandlerWrapper<impl Fn(&T, W) + 'static, T> {
        let f = move |arg: &T, env: W| self(arg, env);
        HandlerWrapper {
            f,
            _marker: std::marker::PhantomData,
        }
    }
}
