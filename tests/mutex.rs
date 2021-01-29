use spinlock::Mutex;

use std::sync::Arc;
use std::thread;

#[test]
fn try_lock_on_unlocked() {
    let mutex = Mutex::new(0);

    assert!(mutex.try_lock().is_some());
}

#[test]
fn try_lock_on_locked() {
    let mutex = Mutex::new(0);

    let _guard = mutex.lock();

    assert!(mutex.try_lock().is_none());
}

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

    thread1.join().unwrap();
    thread2.join().unwrap();

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
        let mut num_fails = 0;
        for _ in 0..1_000_000 {
            if let Some(mut c) = count2.try_lock() {
                *c += acc;
                acc = 1;
            } else {
                acc += 1;
                num_fails += 1;
            }
        }

        // Handle case where try_lock failed in last loop iteration
        let mut c = count2.lock();
        if *c < 2_000_000 {
            *c += acc - 1;
        }

        // Should fail at least one time
        assert!(num_fails > 0);
    });

    thread1.join().unwrap();
    thread2.join().unwrap();

    assert_eq!(*count.lock(), 2_000_000);
}
