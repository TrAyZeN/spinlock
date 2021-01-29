use core::cell::UnsafeCell;
use core::hint;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicIsize, Ordering};

/// A reader-writer lock.
///
/// This primitive allows multiple readers or one unique writer.
#[derive(Debug)]
pub struct RwLock<T> {
    // Inner data contained in the RwLock.
    data: UnsafeCell<T>,

    // The lock
    // lock > 0 => number of shared read access held
    // lock == 0 => no access held
    // lock == -1 => exclusive write access is held
    //
    // Note: This is not optimized we are only using -1, 0, and positive values
    // It could be improved by using a bit to represent exclusive write access
    lock: AtomicIsize,
}

impl<T> RwLock<T> {
    /// Creates a new `RwLock<T>` which is unlocked.
    ///
    /// # Examples
    /// ```
    /// use spinlock::RwLock;
    ///
    /// let lock = RwLock::new(1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            lock: AtomicIsize::new(0),
        }
    }

    /// Acquires the rwlock with shared read access,
    /// blocking the thread until it is available.
    ///
    /// This function blocks the current thread by spinning
    /// if write access is held until it is released.
    ///
    /// ```
    /// use spinlock::RwLock;
    /// use std::thread;
    /// use std::sync::Arc;
    ///
    /// let rwlock = Arc::new(RwLock::new(1));
    /// let r = rwlock.clone();
    ///
    /// thread::spawn(move || {
    ///     assert_eq!(*r.read(), 1);
    /// }).join();
    /// ```
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        loop {
            // Gets the current valid lock value ie not
            // exclusive write access held.
            let lock = loop {
                let lock = self.lock.load(Ordering::Relaxed);
                if lock >= 0 {
                    break lock;
                }

                hint::spin_loop();
            };

            if self
                .lock
                .compare_exchange(lock, lock + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return RwLockReadGuard::new(self);
            }
        }
    }

    /// Tries to acquire the rwlock with shared read access. If the lock is not available returns `None`.
    ///
    /// This function does not block the current thread.
    ///
    /// # Examples
    /// ```
    /// use spinlock::RwLock;
    ///
    /// let rwlock = RwLock::new(1);
    ///
    /// assert_eq!(*rwlock.try_read().unwrap(), 1);
    /// ```
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        if self.lock.fetch_add(1, Ordering::Acquire) >= 0 {
            Some(RwLockReadGuard::new(self))
        } else {
            self.lock.fetch_sub(1, Ordering::Release);
            None
        }
    }

    /// Acquires the rwlock with exclusive write access,
    /// blocking the thread until it is available.
    ///
    /// This function blocks the current thread by spinning
    /// if any read access is held until it is released.
    ///
    /// ```
    /// use spinlock::RwLock;
    /// use std::thread;
    /// use std::sync::Arc;
    ///
    /// let rwlock = Arc::new(RwLock::new(1));
    /// let r = rwlock.clone();
    ///
    /// thread::spawn(move || {
    ///     *r.write() = 42;
    /// }).join();
    /// assert_eq!(*rwlock.read(), 42);
    /// ```
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        loop {
            if self
                .lock
                .compare_exchange(0, -1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return RwLockWriteGuard::new(self);
            }

            while self.lock.load(Ordering::Relaxed) != 0 {
                hint::spin_loop();
            }
        }
    }

    /// Tries to acquire the rwlock with exclusive write access. If the lock is not available returns `None`.
    ///
    /// This function does not block the current thread.
    ///
    /// # Examples
    /// ```
    /// use spinlock::RwLock;
    ///
    /// let rwlock = RwLock::new(1);
    ///
    /// let mut g = rwlock.try_write().unwrap();
    /// *g = 2;
    /// // Release the guard otherwise it would deadlock
    /// drop(g);
    ///
    /// assert_eq!(*rwlock.read(), 2);
    /// ```
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        self.lock
            .compare_exchange(0, -1, Ordering::Acquire, Ordering::Relaxed)
            .map_or(None, |_| Some(RwLockWriteGuard::new(self)))
    }
}

impl<T: Default> Default for RwLock<T> {
    /// Creates a new `RwLock<T>` which is unlocked containing the default of `T`.
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

// SAFETY: The locking mechanism ensures that only one write access
// or multiple read access are possible so it is safe to implement Sync
// for a `T` that is Sync itself.
unsafe impl<T: Sync> Sync for RwLock<T> {}

/// Guard structure used to release the shared read access when dropped.
///
/// This structure is created by [`read`](self::RwLock::read) and
/// [`try_read`](self::RwLock::try_read) on [`RwLock`](self::RwLock).
#[derive(Debug)]
pub struct RwLockReadGuard<'rwlock, T> {
    rwlock: &'rwlock RwLock<T>,
}

impl<'rwlock, T> RwLockReadGuard<'rwlock, T> {
    /// Creates a new `RwLockReadGuard<'rwlock, T>` from a given `RwLock<T>`.
    #[inline]
    #[must_use]
    const fn new(rwlock: &'rwlock RwLock<T>) -> Self {
        Self { rwlock }
    }
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: ReadGuards are only created when no WriteGuard
        // are held so the data can't be modified while a ReadGuard is held
        // so it is safe to get a reference to the data for the lifetime of
        // the guard.
        unsafe { &*self.rwlock.data.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.rwlock.lock.fetch_sub(1, Ordering::Release);
    }
}

// Prevents the read guard from being moved to an other thread.
impl<T> !Send for RwLockReadGuard<'_, T> {}

/// Guard structure used to release the excusive write access when dropped.
///
/// This structure is created by [`write`](self::RwLock::write) and
/// [`try_write`](self::RwLock::try_write) on [`RwLock`](self::RwLock).
#[derive(Debug)]
pub struct RwLockWriteGuard<'rwlock, T> {
    rwlock: &'rwlock RwLock<T>,
}

impl<'rwlock, T> RwLockWriteGuard<'rwlock, T> {
    /// Creates a new `RwLockWriteGuard<'rwlock, T>` from a given `RwLock<T>`.
    #[inline]
    #[must_use]
    const fn new(rwlock: &'rwlock RwLock<T>) -> Self {
        Self { rwlock }
    }
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: A WriteGuard is created only if no other guard is held
        // so it is safe to give a reference to the data for the lifetime of
        // the guard.
        unsafe { &*self.rwlock.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: A WriteGuard is created only if no other guard is held
        // so it is safe to give a mutable reference to the data for the
        // lifetime of the guard.
        unsafe { &mut *self.rwlock.data.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // There could only be one WriteGuard and no other guards
        // so we can directly store 0.
        self.rwlock.lock.store(0, Ordering::Release);
    }
}

// Prevents the write guard from being moved to an other thread.
impl<T> !Send for RwLockWriteGuard<'_, T> {}
