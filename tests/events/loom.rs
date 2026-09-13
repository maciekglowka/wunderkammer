use std::sync::atomic::Ordering;

use wunderkammer::prelude::BusHandle;

use loom::sync::{atomic::AtomicU32, Arc};

#[test]
fn synchronize_buses() {
    loom::model(|| {
        // This test will simply check if all the handlers have been executed.
        let counter = Arc::new(AtomicU32::new(0));

        // The `world` context does not matter here.
        let bus: BusHandle<()> = BusHandle::default();

        let thread_no = 2;
        // Create buses first so the get the event queued.
        let to_spawn = (0..thread_no)
            .map(|_| bus.spawn_handle())
            .collect::<Vec<BusHandle<()>>>();

        // Send the event before spawning the threads.
        bus.send(3_u32);
        bus.send(4_u32);

        let handles = to_spawn
            .into_iter()
            .map(|mut child| {
                let counter = Arc::clone(&counter);
                let add = move |ev: &u32| {
                    counter.fetch_add(*ev, Ordering::Relaxed);
                    Ok(())
                };

                loom::thread::spawn(move || {
                    // Add system to each bus.
                    child.add_handler(add);
                    // Pass fake context.
                    child.step_all_current(&mut ())
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(thread_no * (3 + 4), counter.load(Ordering::Relaxed));
    });
}
