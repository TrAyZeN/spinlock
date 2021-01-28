//! A `no_std` Spinlock implementation.
#![warn(
    missing_docs,
    rust_2018_idioms,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(clippy::module_name_repetitions)]
#![feature(negative_impls)]
#![no_std]

mod mutex;

pub use mutex::Mutex;
