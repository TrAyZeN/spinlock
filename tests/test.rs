use spinlock::Mutex;

use std::sync::Arc;
use std::thread;

#[test]
fn two_threads_count() {
    let count = Arc::new(Mutex::new(0));

    let count1 = count.clone();
    let thread1 = thread::spawn(move || {
        for _ in 0..1_000_000 {
            *count1.lock() += 1;
        }
    });

    let count2 = count.clone();
    let thread2 = thread::spawn(move || {
        for _ in 0..1_000_000 {
            *count2.lock() += 1;
        }
    });

    let _ = thread1.join();
    let _ = thread2.join();

    assert_eq!(*count.lock(), 2_000_000);
}

#[test]
fn two_threads_try_count() {
    let count = Arc::new(Mutex::new(0));

    let count1 = count.clone();
    let thread1 = thread::spawn(move || {
        for _ in 0..1_000_000 {
            *count1.lock() += 1;
        }
    });

    let count2 = count.clone();
    let thread2 = thread::spawn(move || {
        let mut acc = 1;
        for _ in 0..1_000_000 {
            if let Some(mut c) = count2.try_lock() {
                *c += acc;
                acc = 1;
            } else {
                acc += 1;
            }
        }
    });

    let _ = thread1.join();
    let _ = thread2.join();

    assert_eq!(*count.lock(), 2_000_000);
}
