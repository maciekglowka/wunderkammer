use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
};

mod context;
pub mod markers;

use context::Context;

struct ScheduledEvent(TypeId, Arc<dyn Any + Send + Sync>);
impl ScheduledEvent {
    fn new<T: Send + Sync + 'static>(ev: T) -> Self {
        ScheduledEvent(TypeId::of::<T>(), Arc::new(ev))
    }
}

pub type HandlerResult = Result<(), Box<dyn std::error::Error>>;

pub struct BusHandle<C, M = markers::Mut> {
    queue: Arc<Mutex<EventQueue>>,
    handlers: HashMap<TypeId, Vec<Box<dyn Handler<C, M> + Send>>>,
    front: Arc<AtomicUsize>,
}
impl<C: Send, M: Send> BusHandle<C, M> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn spawn_handle<D, N>(&self) -> BusHandle<D, N> {
        let front = self.queue.lock().unwrap().subscribe();
        BusHandle {
            queue: self.queue.clone(),
            handlers: HashMap::new(),
            front,
        }
    }
    pub fn add_handler<F, T, N>(&mut self, handler: F)
    where
        C: Context<M>,
        F: IntoHandler<F, T, C, M, N>,
        T: Send + Sync + 'static,
    {
        let wrapper = handler.wrap();
        self.handlers
            .entry(TypeId::of::<T>())
            .or_insert(vec![])
            .push(Box::new(wrapper));
    }
    pub fn send<T: Sync + Send + 'static>(&self, ev: T) {
        self.queue
            .lock()
            .unwrap()
            .inner
            .push_back(ScheduledEvent::new(ev));
    }
    pub fn step<'a>(&mut self, mut cx: C::Borrow<'a>) -> bool
    where
        C: Context<M>,
    {
        let (type_id, arg) = match self
            .queue
            .lock()
            .unwrap()
            .inner
            .get(self.front.load(Ordering::Relaxed))
        {
            Some(ScheduledEvent(t, a)) => (*t, Arc::clone(a)),
            None => return false,
        };

        self.front.fetch_add(1, Ordering::Relaxed);
        let mut sender = EventSender::default();

        if let Some(handlers) = self.handlers.get(&type_id) {
            for h in handlers.iter() {
                if let Err(e) = h.handle(&*arg, C::borrow(&mut cx), &mut sender) {
                    #[cfg(feature = "log")]
                    log::debug!("Handler failed: {e}");
                }
            }
        };

        let mut queue = self.queue.lock().unwrap();
        queue.synchronize();
        queue.inner.extend(sender.0);
        true
    }
    pub fn step_all_current<'a>(&mut self, mut cx: C::Borrow<'a>) -> bool
    where
        C: Context<M>,
    {
        let events = self
            .queue
            .lock()
            .unwrap()
            .inner
            .range(self.front.load(Ordering::Relaxed)..)
            .map(|ScheduledEvent(t, a)| (*t, Arc::clone(a)))
            .collect::<Vec<_>>();

        if events.is_empty() {
            return false;
        }

        self.front.fetch_add(events.len(), Ordering::Relaxed);
        let mut sender = EventSender::default();

        for (type_id, arg) in events.iter() {
            let Some(handlers) = self.handlers.get(type_id) else {
                continue;
            };
            for h in handlers.iter() {
                if let Err(e) = h.handle(&**arg, C::borrow(&mut cx), &mut sender) {
                    #[cfg(feature = "log")]
                    log::debug!("Handler failed: {e}");
                }
            }
        }

        let mut queue = self.queue.lock().unwrap();
        queue.synchronize();
        queue.inner.extend(sender.0);
        true
    }
}
impl<C, M> Default for BusHandle<C, M> {
    fn default() -> Self {
        let mut queue = EventQueue::default();
        let front = queue.subscribe();
        BusHandle {
            queue: Arc::new(Mutex::new(queue)),
            handlers: HashMap::new(),
            front,
        }
    }
}

pub trait EventDispatcher {
    fn send<T: Send + Sync + 'static>(&mut self, ev: T);
}

impl<C: Send, M: Send> EventDispatcher for BusHandle<C, M> {
    fn send<T: Send + Sync + 'static>(&mut self, ev: T) {
        BusHandle::send(self, ev);
    }
}

#[derive(Default)]
pub struct EventSender(Vec<ScheduledEvent>);
impl EventDispatcher for EventSender {
    fn send<T: Send + Sync + 'static>(&mut self, ev: T) {
        self.0.push(ScheduledEvent::new(ev));
    }
}

#[derive(Default)]
struct EventQueue {
    inner: VecDeque<ScheduledEvent>,
    subscriber_fronts: Vec<Weak<AtomicUsize>>,
}
impl EventQueue {
    fn subscribe(&mut self) -> Arc<AtomicUsize> {
        let front = Arc::new(AtomicUsize::new(self.inner.len()));
        self.subscriber_fronts.push(Arc::downgrade(&front));
        front
    }
    fn synchronize(&mut self) {
        // The typical subscriber count will be quite low (e.g. 3 - 5kj)
        // Keeping double iteration for simplicity.

        let mut min = usize::MAX;

        for i in (0..self.subscriber_fronts.len()).rev() {
            match self.subscriber_fronts[i].upgrade() {
                None => {
                    // Dropped, remove.
                    self.subscriber_fronts.remove(i);
                }
                Some(f) => {
                    min = min.min(f.load(Ordering::Relaxed));
                }
            }
        }

        if min == 0 || min == usize::MAX {
            // No synchronization is needed.
            return;
        }

        self.subscriber_fronts.iter().for_each(|f| {
            if let Some(f) = f.upgrade() {
                // Subtraction should be safe as fronts are only increased
                // in other parts of the code.
                f.fetch_sub(min, Ordering::Relaxed);
            }
        });
        self.inner.drain(..min);
    }
}

trait Handler<C: Context<M> + Send, M: Send> {
    fn handle<'a>(
        &self,
        arg: &dyn Any,
        cx: C::Borrow<'a>,
        sender: &mut EventSender,
    ) -> HandlerResult;
}

pub struct HandlerWrapper<F, T> {
    f: F,
    _marker: std::marker::PhantomData<T>,
    f_name: &'static str,
}

impl<F, T, C, M> Handler<C, M> for HandlerWrapper<F, T>
where
    C: Context<M> + Send,
    M: Send,
    F: Fn(&T, C::Borrow<'_>, &mut EventSender) -> HandlerResult,
    T: Send + Sync + 'static,
{
    fn handle<'a>(
        &self,
        arg: &dyn Any,
        cx: C::Borrow<'a>,
        sender: &mut EventSender,
    ) -> HandlerResult {
        #[cfg(feature = "log")]
        log::debug!(
            "Executing handler {} for: {}",
            self.f_name,
            std::any::type_name::<T>()
        );
        let arg = arg.downcast_ref().unwrap();
        (self.f)(arg, cx, sender)
    }
}

pub trait IntoHandler<F, T, C, M, N>
where
    C: Context<M> + Send,
    M: Send,
    T: Send + Sync + 'static,
{
    fn wrap(
        self,
    ) -> HandlerWrapper<
        impl for<'a> Fn(&T, C::Borrow<'a>, &mut EventSender) -> HandlerResult + Send + 'static,
        T,
    >;
}

impl<F, T, C, M> IntoHandler<F, T, C, M, markers::EventOnly> for F
where
    C: Context<M> + Send,
    M: Send,
    F: Fn(&T) -> HandlerResult + Send + 'static,
    T: Send + Sync + 'static,
{
    fn wrap(
        self,
    ) -> HandlerWrapper<
        impl for<'a> Fn(&T, C::Borrow<'a>, &mut EventSender) -> HandlerResult + Send + 'static,
        T,
    > {
        let f_name = std::any::type_name_of_val(&self);
        let f = move |arg: &T, _: C::Borrow<'_>, _: &mut EventSender| self(arg);
        HandlerWrapper {
            f,
            f_name,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<F, T, C, M> IntoHandler<F, T, C, M, markers::WithContext> for F
where
    C: Context<M> + Send,
    M: Send,
    F: Fn(&T, C::Borrow<'_>) -> HandlerResult + Send + 'static,
    T: Send + Sync + 'static,
{
    fn wrap(
        self,
    ) -> HandlerWrapper<
        impl for<'a> Fn(&T, C::Borrow<'a>, &mut EventSender) -> HandlerResult + Send + 'static,
        T,
    > {
        let f_name = std::any::type_name_of_val(&self);
        let f = move |arg: &T, cx: C::Borrow<'_>, _: &mut EventSender| self(arg, cx);
        HandlerWrapper {
            f,
            f_name,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<F, T, C, M> IntoHandler<F, T, C, M, markers::WithSender> for F
where
    C: Context<M> + Send,
    M: Send,
    F: Fn(&T, &mut EventSender) -> HandlerResult + Send + 'static,
    T: Send + Sync + 'static,
{
    fn wrap(
        self,
    ) -> HandlerWrapper<
        impl for<'a> Fn(&T, C::Borrow<'a>, &mut EventSender) -> HandlerResult + Send + 'static,
        T,
    > {
        let f_name = std::any::type_name_of_val(&self);
        let f = move |arg: &T, _: C::Borrow<'_>, sender: &mut EventSender| self(arg, sender);
        HandlerWrapper {
            f,
            f_name,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<F, T, C, M> IntoHandler<F, T, C, M, markers::WithContextAndSender> for F
where
    C: Context<M> + Send,
    M: Send,
    F: Fn(&T, C::Borrow<'_>, &mut EventSender) -> HandlerResult + Send + 'static,
    T: Send + Sync + 'static,
{
    fn wrap(
        self,
    ) -> HandlerWrapper<
        impl for<'a> Fn(&T, C::Borrow<'a>, &mut EventSender) -> HandlerResult + Send + 'static,
        T,
    > {
        HandlerWrapper {
            f_name: std::any::type_name_of_val(&self),
            f: self,
            _marker: std::marker::PhantomData,
        }
    }
}
