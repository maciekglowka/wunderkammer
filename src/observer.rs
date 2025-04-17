use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock, Weak,
    },
};

pub struct ObservableQueue<T> {
    queue: Arc<RwLock<VecDeque<T>>>,
    observers: Vec<Weak<AtomicUsize>>,
}
impl<T> ObservableQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            observers: Vec::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        // do not store data when no receivers
        if self.observers.is_empty() {
            return;
        };

        {
            let mut queue = self.queue.write().unwrap();
            queue.push_back(value);
        }
        self.synchronize();
    }

    pub fn subscribe(&mut self) -> Observer<T> {
        let front = Arc::new(AtomicUsize::new(self.queue.read().unwrap().len()));
        self.observers.push(Arc::downgrade(&front));
        Observer {
            front,
            queue: Arc::downgrade(&self.queue),
        }
    }

    fn synchronize(&mut self) {
        let mut queue = self.queue.write().unwrap();
        // purge observers
        self.observers.retain(|a| a.strong_count() > 0);

        // get minimal front
        let mut new_front = self
            .observers
            .iter()
            .filter_map(|a| a.upgrade())
            .map(|a| a.load(Ordering::Relaxed))
            .min()
            .unwrap_or(usize::MAX);

        new_front = new_front.min(queue.len());

        for front in self.observers.iter().filter_map(|a| a.upgrade()) {
            // shift fronts by the amount popped
            front.fetch_sub(new_front, Ordering::Relaxed);
        }

        let _ = queue.drain(..new_front);
    }
}

pub struct Observer<T> {
    front: Arc<AtomicUsize>,
    queue: Weak<RwLock<VecDeque<T>>>,
}
impl<T> Observer<T> {
    pub fn map_next<U>(&self, f: impl Fn(&T) -> U) -> Option<U> {
        let r = self.queue.upgrade()?;
        let queue = r.read().unwrap();

        let next = queue.get(self.front.load(Ordering::Relaxed))?;
        self.front.fetch_add(1, Ordering::Relaxed);
        Some(f(next))
    }
}
impl<T: Clone> Observer<T> {
    pub fn next(&self) -> Option<T> {
        let r = self.queue.upgrade()?;
        let queue = r.read().unwrap();

        let next = queue.get(self.front.load(Ordering::Relaxed))?;
        self.front.fetch_add(1, Ordering::Relaxed);
        Some(next.clone())
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_next_single() {
        let mut queue = ObservableQueue::new();
        let observer = queue.subscribe();

        queue.push(3);
        queue.push(12);

        assert_eq!(observer.next(), Some(3));
        queue.synchronize();
        assert_eq!(queue.queue.read().unwrap().len(), 1);
        assert_eq!(observer.next(), Some(12));
        queue.synchronize();
        assert_eq!(queue.queue.read().unwrap().len(), 0);
    }

    #[test]
    fn test_map_next_single() {
        let mut queue = ObservableQueue::new();
        let observer = queue.subscribe();

        queue.push(3);
        queue.push(12);

        assert_eq!(observer.map_next(|a| *a), Some(3));
        queue.synchronize();
        assert_eq!(queue.queue.read().unwrap().len(), 1);
        assert_eq!(observer.map_next(|a| *a), Some(12));
        queue.synchronize();
        assert_eq!(queue.queue.read().unwrap().len(), 0);
    }

    #[test]
    fn test_next_many() {
        let mut queue = ObservableQueue::new();
        let observers = (0..3).map(|_| queue.subscribe()).collect::<Vec<_>>();

        queue.push(3);
        queue.push(12);
        queue.push(2);

        assert_eq!(observers[0].next(), Some(3));
        assert_eq!(observers[0].next(), Some(12));

        assert_eq!(observers[1].next(), Some(3));

        queue.synchronize();
        // no item should be removed yet as observers[2] still has not read
        assert_eq!(queue.queue.read().unwrap().len(), 3);

        assert_eq!(observers[0].next(), Some(2));
        assert_eq!(observers[2].next(), Some(3));

        queue.synchronize();
        assert_eq!(queue.queue.read().unwrap().len(), 2);
    }

    #[test]
    fn test_map_next_many() {
        let mut queue = ObservableQueue::new();
        let observers = (0..3).map(|_| queue.subscribe()).collect::<Vec<_>>();

        queue.push(3);
        queue.push(12);
        queue.push(2);

        assert_eq!(observers[0].map_next(|a| *a), Some(3));
        assert_eq!(observers[0].map_next(|a| *a), Some(12));

        assert_eq!(observers[1].map_next(|a| *a), Some(3));

        queue.synchronize();
        // no item should be removed yet as observers[2] still has not read
        assert_eq!(queue.queue.read().unwrap().len(), 3);

        assert_eq!(observers[0].map_next(|a| *a), Some(2));
        assert_eq!(observers[2].map_next(|a| *a), Some(3));

        queue.synchronize();
        assert_eq!(queue.queue.read().unwrap().len(), 2);
    }

    #[test]
    fn test_next_after() {
        let mut queue = ObservableQueue::new();

        queue.push(3);
        queue.push(12);

        let observer_0 = queue.subscribe();

        queue.push(1);

        let observer_1 = queue.subscribe();

        assert_eq!(observer_0.next(), Some(1));
        assert_eq!(observer_1.next(), None);
    }

    #[test]
    fn test_map_next_after() {
        let mut queue = ObservableQueue::new();

        queue.push(3);
        queue.push(12);

        let observer_0 = queue.subscribe();

        queue.push(1);

        let observer_1 = queue.subscribe();

        assert_eq!(observer_0.map_next(|a| *a), Some(1));
        assert_eq!(observer_1.map_next(|a| *a), None);
    }

    #[test]
    fn test_drop_observer() {
        let mut queue = ObservableQueue::new();
        let observer = queue.subscribe();

        queue.push(3);
        queue.push(12);

        drop(observer);
        queue.synchronize();
        assert!(queue.observers.is_empty());
        assert!(queue.queue.read().unwrap().is_empty());
    }
}
