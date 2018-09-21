//! Misc utilities for working with the `tokio`.
//!
extern crate tokio_channel;
extern crate tokio;

pub mod service;
mod never;

pub use never::Never;

