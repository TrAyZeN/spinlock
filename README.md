<h1 align="center">
    spinlock
</h1>

![CI](https://github.com/trayzen/spinlock/workflows/CI/badge.svg)

> no_std synchonization primitives using spinlock.

**⚠️ Disclaimer ⚠️**: This implementation is for learning purposes if you want to use a spinlock use the crate [`spin-rs`](https://github.com/mvdnes/spin-rs) instead.

## What is a spinlock ?
> In software engineering, a spinlock is a lock which causes a thread trying to acquire it to simply wait in a loop ("spin") while repeatedly checking if the lock is available.

*From [wikipedia](https://en.wikipedia.org/wiki/Spinlock).*

## Roadmap
- [x] Mutex
- [x] RwLock
- [ ] Handle panicking

## Example
```rust
use std::thread;
use std::sync::Arc;

use spinlock::Mutex;

let count = Arc::new(Mutex::new(0));

let count1 = Arc::clone(&count);
let _ = thread::spawn(move || {
    *count1.lock() += 1;
}).join();

assert_eq!(*count.lock(), 1);
```

## Useful links
- [Correctly implementing a spinlock in C++](https://rigtorp.se/spinlock/)
- [The black art of concurrency](https://www.internalpointers.com/post-group/black-art-concurrency)
- [Nomicon chapter on concurrency](https://doc.rust-lang.org/nomicon/concurrency.html)

## License
This project is licensed under [MIT License](https://github.com/TrAyZeN/spinlock/blob/master/LICENSE).
