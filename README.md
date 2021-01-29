<h1 align="center">
    spinlock
</h1>

> no_std synchonization primitives using spinlock.

**⚠️ Disclaimer ⚠️**: This implementation is for learning purposes if you want to use a spinlock use the crate [`spin-rs`](https://github.com/mvdnes/spin-rs) instead.

## What is a spinlock ?
> In software engineering, a spinlock is a lock which causes a thread trying to acquire it to simply wait in a loop ("spin") while repeatedly checking if the lock is available.

*From [wikipedia](https://en.wikipedia.org/wiki/Spinlock).*

## Roadmap
- [x] Mutex
- [x] RwLock
- [ ] Handle panicking

## Useful links
- [Correctly implementing a spinlock in C++](https://rigtorp.se/spinlock/)
- [The black art of concurrency](https://www.internalpointers.com/post-group/black-art-concurrency)
- [Nomicon chapter on concurrency](https://doc.rust-lang.org/nomicon/concurrency.html)
