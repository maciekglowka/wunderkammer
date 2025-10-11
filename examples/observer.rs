use wunderkammer::prelude::*;

#[derive(Clone)]
struct NumberEvent(u32);

// Handler
fn is_even(ev: &mut NumberEvent) -> EventResult {
    match ev.0 % 2 {
        0 => Ok(()),
        _ => Err(EventError::Break),
    }
}

fn main() {
    let mut scheduler: Scheduler<()> = Scheduler::new();
    let observer = scheduler.observe::<NumberEvent>();

    let _ = std::thread::spawn(move || loop {
        if let Some(ev) = observer.next() {
            println!("{} is even", ev.0);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    });

    scheduler.add_system(is_even);

    scheduler.send(NumberEvent(2));
    scheduler.send(NumberEvent(3));
    scheduler.send(NumberEvent(5));
    scheduler.send(NumberEvent(4));

    // Process all the events
    while scheduler.step(&mut ()) {}

    // Wait for handlers.
    std::thread::sleep(std::time::Duration::from_millis(500));
}
