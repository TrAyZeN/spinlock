use core::cell::UnsafeCell;
use core::hint;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A mutual exclusion synchronization primitive.
///
/// This primitive allows only one thread to access the data at a time.
/// It is using Spinlock as locking mechanism causing the thread trying
/// to acquire the lock to spin.
///
/// This structure provides interior mutability and prevents multiple
/// threads to access the data at the same time.
#[derive(Debug)]
pub struct Mutex<T> {
    // Inner data contained in the mutex.
    data: UnsafeCell<T>,
    // Is the lock held by a thread.
    lock: AtomicBool,
}

impl<T> Mutex<T> {
    /// Creates a new `Mutex<T>` which is unlocked.
    ///
    /// # Examples
    /// ```
    /// use spinlock::Mutex;
    ///
    /// let mutex = Mutex::new(1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            lock: AtomicBool::new(false),
        }
    }

    /// Acquires the lock, blocking the current thread until the lock is available.
    ///
    /// This functions block the current thread until the lock is available.
    ///
    /// # Examples
    /// ```
    /// use spinlock::Mutex;
    /// use std::thread;
    /// use std::sync::Arc;
    ///
    /// let mutex = Arc::new(Mutex::new(1));
    /// let m = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     *m.lock() = 42;
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock(), 42);
    /// ```
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // To reduce the cache coherency traffic we spin on an atomic load which does
        // not requires write access to the cache line (as opposed to compare_and_swap).
        loop {
            // Memory order acquire is used to make sure no reordering happens after it.
            if !self.lock.swap(true, Ordering::Acquire) {
                return MutexGuard::new(self);
            }

            while self.lock.load(Ordering::Relaxed) {
                // Hints the CPU that we are in a busy-wait spin loop, so the CPU can
                // optimized its behavior.
                hint::spin_loop();
            }
        }
    }

    /// Tries to acquire the lock. If the lock is not available returns `None`.
    ///
    /// This function does not block the current thread.
    ///
    /// # Examples
    /// ```
    /// use spinlock::Mutex;
    ///
    /// let mutex = Mutex::new(1);
    /// assert_eq!(*mutex.try_lock().unwrap(), 1);
    /// ```
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if !self.lock.load(Ordering::Relaxed) && !self.lock.swap(true, Ordering::Acquire) {
            Some(MutexGuard::new(self))
        } else {
            None
        }
    }

    /// UNSAFE: forcing to unlock while a guard is still held may allow to have mutliple guards.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    unsafe fn unlock(&self) {
        // Memory order acquire is used to make sure no reordering happens before it.
        self.lock.store(false, Ordering::Release);
    }
}

impl<T: Default> Default for Mutex<T> {
    /// Creates a `Mutex<T>` which is unlocked containing the default of `T`.
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

// SAFETY: It is safe to impl Sync since the locking mechanism ensures the synchronization.
unsafe impl<T: Sync> Sync for Mutex<T> {}

/// This structure is created by calling [`lock`](self::Mutex::lock)
/// or [`try_lock`](self::Mutex::try_lock) on [`Mutex`](self::Mutex).
#[derive(Debug)]
pub struct MutexGuard<'mutex, T> {
    mutex: &'mutex Mutex<T>,
}

impl<'mutex, T> MutexGuard<'mutex, T> {
    /// Creates a `MutexGuard<'mutex, T>` of a given Mutex.
    #[inline]
    #[must_use]
    const fn new(mutex: &'mutex Mutex<T>) -> Self {
        Self { mutex }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: A guard is only created if no one holds the lock meaning that
        // no one else can modify the data so it is safe to get reference to the
        // data for the lifetime of the guard.
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: A guard is only created if no one holds the lock meaning that
        // no one else can modify the data so it is safe to get a mutable reference
        // to the data for the lifetime of the guard.
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: It is only possible that one guard exists for a certain mutex
        // which is the current one so it is safe to unlock the mutex when the
        // guard gets dropped.
        unsafe { self.mutex.unlock() }
    }
}

/// Prevents the guard from being sent to another thread.
impl<T> !Send for MutexGuard<'_, T> {}

unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}
