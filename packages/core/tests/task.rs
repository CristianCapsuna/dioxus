//! Verify that tasks get polled by the virtualdom properly, and that we escape wait_for_work safely

#[cfg(not(miri))]
#[tokio::test]
async fn it_works() {
    use dioxus::prelude::*;
    use std::{sync::atomic::AtomicUsize, time::Duration};

    static POLL_COUNT: AtomicUsize = AtomicUsize::new(0);

    fn app() -> Element {
        once(|| {
            spawn(async {
                for x in 0..10 {
                    tokio::time::sleep(Duration::from_micros(50)).await;
                    POLL_COUNT.fetch_add(x, std::sync::atomic::Ordering::Relaxed);
                }
            });

            spawn(async {
                for x in 0..10 {
                    tokio::time::sleep(Duration::from_micros(25)).await;
                    POLL_COUNT.fetch_add(x * 2, std::sync::atomic::Ordering::Relaxed);
                }
            });
        });

        render!({ () })
    }

    let mut dom = VirtualDom::new(app);

    let _ = dom.rebuild(&mut dioxus_core::NoOpMutations);

    tokio::select! {
        _ = dom.wait_for_work() => {}
        _ = tokio::time::sleep(Duration::from_millis(500)) => {}
    };

    // By the time the tasks are finished, we should've accumulated ticks from two tasks
    // Be warned that by setting the delay to too short, tokio might not schedule in the tasks
    assert_eq!(
        POLL_COUNT.fetch_add(0, std::sync::atomic::Ordering::Relaxed),
        135
    );
}
