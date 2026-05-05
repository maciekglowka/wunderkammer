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
    pub fn step(&mut self, env: &mut W) {
        let mut queue = self.queue.lock().unwrap();

        let Some(front) = queue.inner.get(self.front.load(Ordering::Relaxed)) else {
            return;
        };
        self.front.fetch_add(1, Ordering::Relaxed);

        if let Some(handlers) = self.handlers.get(&front.0) {
            handlers.iter().for_each(|h| h.handle(&*front.1, env));
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

trait Handler<W> {
    fn handle(&self, arg: &dyn Any, env: &mut W);
}

struct HandlerWrapper<F, T> {
    f: F,
    _marker: std::marker::PhantomData<T>,
}

impl<F, T, W> Handler<W> for HandlerWrapper<F, T>
where
    F: Fn(&T, &mut W),
    T: 'static,
{
    fn handle(&self, arg: &dyn Any, env: &mut W) {
        let arg = arg.downcast_ref().unwrap();
        (self.f)(arg, env);
    }
}

trait IntoWrapper<F, T, W> {
    fn wrap(self) -> HandlerWrapper<impl Fn(&T, &mut W) + 'static, T>;
}
impl<F, T, W> IntoWrapper<F, T, W> for F
where
    F: Fn(&T, &mut W) + 'static,
    T: 'static,
{
    fn wrap(self) -> HandlerWrapper<impl Fn(&T, &mut W) + 'static, T> {
        let f = move |arg: &T, env: &mut W| self(arg, env);
        HandlerWrapper {
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

// trait HandlerSetErased {
//     type Env;
//     // fn add_handler(&mut self, handler: Box<dyn Any>);
//     fn handle(&self, event: &dyn Any, env: Self::Env);
// }

// struct HandlerSet<T> {
//     // handlers: Vec<EventHandler<T, W>>,
// }
// impl<T> HandlerSet<T> {
//     fn new() -> Self {
//         Self {
//             // handlers: Vec::new(),
//         }
//     }
// }
// impl<T: 'static, W> HandlerSetErased for HandlerSet<T> {
//     type Env = W;
//     // fn add_handler(&mut self, handler: Box<dyn Any>) {
//     //     let handler = *handler.downcast().unwrap();
//     //     self.handlers.push(handler);
//     // }
//     fn handle(&self, event: &dyn Any, mut env: W) {
//         // let event = event.downcast_ref().unwrap();
//         // self.handlers.iter().for_each(|h| h.execute(event, env.a()));
//     }
// }

// pub struct EventHandler<T, W>(pub Box<dyn Fn(&T, &mut W)>);
// pub struct EventHandler<T, W>(fn(&T, &mut W) -> ());
// impl<T, W> EventHandler<T, W> {
//     fn execute(&self, event: &T, env: &mut W) {
//         self.0(event, env);
//     }
// }

// pub trait IntoHandler<T, W, M> {
//     fn handler(self) -> EventHandler<T, W>;
// }
