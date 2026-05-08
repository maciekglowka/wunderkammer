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

pub struct EventSubscriber<C, M = Mut> {
    queue: Arc<Mutex<EventQueue>>,
    handlers: HashMap<TypeId, Vec<Box<dyn Handler<C, M>>>>,
    front: Arc<AtomicUsize>,
}
impl<C, M> EventSubscriber<C, M> {
    pub fn spawn_subscriber<D, N>(&self) -> EventSubscriber<D, N> {
        let front = self.queue.lock().unwrap().subscribe();
        EventSubscriber {
            queue: self.queue.clone(),
            handlers: HashMap::new(),
            front,
        }
    }
    pub fn add_handler<F, T>(&mut self, handler: F)
    where
        C: Context<M>,
        F: IntoHandler<F, T, C, M>,
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
    pub fn step<'a>(&mut self, mut cx: C::Borrow<'a>)
    where
        C: Context<M>,
    {
        let mut queue = self.queue.lock().unwrap();

        let Some(front) = queue.inner.get(self.front.load(Ordering::Relaxed)) else {
            return;
        };
        self.front.fetch_add(1, Ordering::Relaxed);

        if let Some(handlers) = self.handlers.get(&front.0) {
            for h in handlers.iter() {
                h.handle(&*front.1, C::borrow(&mut cx));
            }
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

pub trait Context<M = Mut> {
    type Borrow<'a>
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b;
}

pub struct Ref;
pub struct Mut;

impl<C> Context for C {
    type Borrow<'a>
        = &'a mut C
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b,
    {
        &mut **cx
    }
}
impl<C> Context<Ref> for C {
    type Borrow<'a>
        = &'a C
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b,
    {
        &**cx
    }
}
impl<C, D> Context<(Mut, Mut)> for (C, D) {
    type Borrow<'a>
        = (&'a mut C, &'a mut D)
    where
        Self: 'a;

    fn borrow<'b, 'a>(cx: &'b mut Self::Borrow<'a>) -> Self::Borrow<'b>
    where
        'a: 'b,
    {
        (&mut *cx.0, &mut *cx.1)
    }
}

trait Handler<C: Context<M>, M> {
    fn handle<'a>(&self, arg: &dyn Any, cx: C::Borrow<'a>);
}

pub struct HandlerWrapper<F, T> {
    f: F,
    _marker: std::marker::PhantomData<T>,
}

impl<F, T, C, M> Handler<C, M> for HandlerWrapper<F, T>
where
    C: Context<M>,
    F: Fn(&T, C::Borrow<'_>),
    T: 'static,
{
    fn handle<'a>(&self, arg: &dyn Any, cx: C::Borrow<'a>) {
        let arg = arg.downcast_ref().unwrap();
        (self.f)(arg, cx);
    }
}

pub trait IntoHandler<F, T, C, M>
where
    C: Context<M>,
{
    fn wrap(self) -> HandlerWrapper<impl for<'a> Fn(&T, C::Borrow<'a>) + 'static, T>;
}
impl<F, T, C, M> IntoHandler<F, T, C, M> for F
where
    C: Context<M>,
    F: Fn(&T, C::Borrow<'_>) + 'static,
    T: 'static,
{
    fn wrap(self) -> HandlerWrapper<impl for<'a> Fn(&T, C::Borrow<'a>) + 'static, T> {
        HandlerWrapper {
            f: self,
            _marker: std::marker::PhantomData,
        }
    }
}
