use spinlock::RwLock;

use std::sync::Arc;
use std::thread;

#[test]
fn multiple_read_guard() {
    let rwlock = RwLock::new(0);

    let _rguard1 = rwlock.read();

    // This should dead lock if two read guards were not possible
    let _rguard2 = rwlock.read();
}

#[test]
fn try_read_on_unlocked() {
    let rwlock = RwLock::new(0);

    assert!(rwlock.try_read().is_some());
}

#[test]
fn try_read_on_locked() {
    let rwlock = RwLock::new(0);

    let _wguard = rwlock.write();

    assert!(rwlock.try_read().is_none());
}

#[test]
fn try_write_on_unlocked() {
    let rwlock = RwLock::new(0);

    assert!(rwlock.try_write().is_some());
}

#[test]
fn try_write_on_locked() {
    let rwlock = RwLock::new(0);

    let _rguard = rwlock.read();

    assert!(rwlock.try_write().is_none());
}

#[test]
fn two_threads_count() {
    let count = Arc::new(RwLock::new(0));

    let count1 = count.clone();
    let thread1 = thread::spawn(move || {
        for _ in 0..1_000_000 {
            *count1.write() += 1;
        }

        assert!(*count1.read() >= 1_000_000);
    });

    let count2 = count.clone();
    let thread2 = thread::spawn(move || {
        for _ in 0..1_000_000 {
            *count2.write() += 1;
        }

        assert!(*count2.read() >= 1_000_000);
    });

    thread1.join().unwrap();
    thread2.join().unwrap();

    assert_eq!(*count.read(), 2_000_000);
}
