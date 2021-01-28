<h1 align="center">
    spinlock
</h1>

> no_std synchonization primitives using spinlock.

**⚠️ Disclaimer ⚠️**: This implementation is for learning purposes if you want to use a spinlock use the crate [`spin-rs`](https://github.com/mvdnes/spin-rs) instead.

## What is a spinlock ?
> In software engineering, a spinlock is a lock which causes a thread trying to acquire it to simply wait in a loop ("spin") while repeatedly checking if the lock is available.
<div style="text-align: right; font-style: italic">
    From <a href="https://en.wikipedia.org/wiki/Spinlock">wikipedia</a>.
</div>

## Roadmap
- [x] Mutex
- [ ] RwLock
- [ ] Handle panicking

## Useful links
- https://rigtorp.se/spinlock/
- https://www.internalpointers.com/post/gentle-introduction-multithreading
- https://doc.rust-lang.org/nomicon/concurrency.html
