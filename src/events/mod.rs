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

#[cfg(feature = "send")]
type StoredHandler<C, M> = Box<dyn Handler<C, M> + Send>;
#[cfg(not(feature = "send"))]
type StoredHandler<C, M> = Box<dyn Handler<C, M>>;

pub struct BusHandle<C, M = markers::Mut> {
    queue: Arc<Mutex<EventQueue>>,
    // handlers: HashMap<TypeId, Vec<Box<dyn Handler<C, M>>>>,
    handlers: HashMap<TypeId, Vec<StoredHandler<C, M>>>,
    front: Arc<AtomicUsize>,
}
impl<C: Context<M> + MaybeSend, M: MaybeSend> BusHandle<C, M> {
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
    pub fn add_handler<F, T, N>(&mut self, handler: F) -> HandlerId
    where
        C: Context<M>,
        F: IntoHandler<F, T, C, M, N>,
        T: Send + Sync + 'static,
    {
        let wrapper = handler.wrap();
        let id = HandlerId {
            arg_id: TypeId::of::<T>(),
            f_id: wrapper.f_id,
        };
        self.handlers
            .entry(TypeId::of::<T>())
            .or_insert(vec![])
            .push(Box::new(wrapper));

        id
    }
    /// Remove a handler by id.
    /// If the same function has been registered more than once
    /// all instances will be removed.
    ///
    /// Returns a number of removed handlers.
    pub fn remove_handler(&mut self, handler_id: HandlerId) -> usize {
        if let Some(entry) = self.handlers.get_mut(&handler_id.arg_id) {
            entry
                .extract_if(.., |h| h.f_id() == handler_id.f_id)
                .count()
        } else {
            0
        }
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
                #[allow(unused_variables)]
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
                #[allow(unused_variables)]
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

impl<C: Context<M> + MaybeSend, M: MaybeSend> EventDispatcher for BusHandle<C, M> {
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

trait Handler<C: Context<M> + MaybeSend, M: MaybeSend> {
    fn handle<'a>(
        &self,
        arg: &dyn Any,
        cx: C::Borrow<'a>,
        sender: &mut EventSender,
    ) -> HandlerResult;
    fn f_id(&self) -> TypeId;
}

#[derive(Clone, Copy, Debug)]
pub struct HandlerId {
    arg_id: TypeId,
    f_id: TypeId,
}

pub struct HandlerWrapper<F, T> {
    f: F,
    _marker: std::marker::PhantomData<T>,
    #[allow(dead_code)]
    f_name: &'static str,
    f_id: TypeId,
}

impl<F, T, C, M> Handler<C, M> for HandlerWrapper<F, T>
where
    C: Context<M> + MaybeSend,
    M: MaybeSend,
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
    fn f_id(&self) -> TypeId {
        self.f_id
    }
}

pub trait IntoHandler<F, T, C, M, N>
where
    C: Context<M> + MaybeSend,
    M: MaybeSend,
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
    C: Context<M> + MaybeSend,
    M: MaybeSend,
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
        let f_id = TypeId::of::<F>();
        let f = move |arg: &T, _: C::Borrow<'_>, _: &mut EventSender| self(arg);
        HandlerWrapper {
            f,
            f_name,
            f_id,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<F, T, C, M> IntoHandler<F, T, C, M, markers::WithContext> for F
where
    C: Context<M> + MaybeSend,
    M: MaybeSend,
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
        let f_id = TypeId::of::<F>();
        let f = move |arg: &T, cx: C::Borrow<'_>, _: &mut EventSender| self(arg, cx);
        HandlerWrapper {
            f,
            f_name,
            f_id,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<F, T, C, M> IntoHandler<F, T, C, M, markers::WithSender> for F
where
    C: Context<M> + MaybeSend,
    M: MaybeSend,
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
        let f_id = TypeId::of::<F>();
        let f = move |arg: &T, _: C::Borrow<'_>, sender: &mut EventSender| self(arg, sender);
        HandlerWrapper {
            f,
            f_name,
            f_id,
            _marker: std::marker::PhantomData,
        }
    }
}
impl<F, T, C, M> IntoHandler<F, T, C, M, markers::WithContextAndSender> for F
where
    C: Context<M> + MaybeSend,
    M: MaybeSend,
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
            f_id: TypeId::of::<F>(),
            f: self,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "send")]
pub trait MaybeSend: Send {}

#[cfg(not(feature = "send"))]
pub trait MaybeSend {}

#[cfg(feature = "send")]
impl<T: Send> MaybeSend for T {}

#[cfg(not(feature = "send"))]
impl<T> MaybeSend for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct World(u32);

    fn add(ev: &u32, w: &mut World) -> HandlerResult {
        w.0 += *ev;
        Ok(())
    }
    fn mul(ev: &u32, w: &mut World) -> HandlerResult {
        w.0 *= *ev;
        Ok(())
    }

    #[test]
    fn add_handler() {
        let mut w = World::default();
        let mut bus: BusHandle<World> = BusHandle::default();
        bus.add_handler(add);

        bus.send(3_u32);
        bus.step(&mut w);
        assert_eq!(w.0, 3);
    }
    #[test]
    fn add_two_handlers() {
        let mut w = World::default();
        let mut bus: BusHandle<World> = BusHandle::default();
        bus.add_handler(add);
        bus.add_handler(mul);

        bus.send(3_u32);
        bus.step(&mut w);
        // Execution order is not guaranteed but in this simple case
        // should match insertion order.
        assert_eq!(w.0, 9);
    }
    /// This test removes the only handler.
    #[test]
    fn remove_handler() {
        let mut w = World::default();
        let mut bus: BusHandle<World> = BusHandle::default();
        let id = bus.add_handler(add);

        bus.send(3_u32);
        bus.step(&mut w);

        let removed = bus.remove_handler(id);
        assert_eq!(1, removed);

        bus.send(3_u32);
        bus.step(&mut w);
        assert_eq!(w.0, 3);
    }
    /// This test removes duplicate handlers.
    #[test]
    fn remove_two_handlers() {
        let mut w = World::default();
        let mut bus: BusHandle<World> = BusHandle::default();

        let id = bus.add_handler(add);
        let _ = bus.add_handler(add);

        bus.send(3_u32);
        bus.step(&mut w);

        let removed = bus.remove_handler(id);
        assert_eq!(2, removed);

        bus.send(3_u32);
        bus.step(&mut w);
        assert_eq!(w.0, 6);
    }
    /// This test removes one handler and checks other is left intact.
    #[test]
    fn remove_handler_keep() {
        let mut w = World::default();
        let mut bus: BusHandle<World> = BusHandle::default();
        let id = bus.add_handler(add);
        bus.add_handler(mul);

        bus.send(3_u32);
        bus.step(&mut w);

        let removed = bus.remove_handler(id);
        assert_eq!(1, removed);

        bus.send(3_u32);
        bus.step(&mut w);
        assert_eq!(w.0, 27);
    }
}
