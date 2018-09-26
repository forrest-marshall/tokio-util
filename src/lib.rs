//! Misc utilities for working with the `tokio`.
//!
extern crate tokio_channel;
extern crate tokio;

#[cfg(feature = "serde-impls")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serde-impls")]
extern crate serde;

pub mod service;
mod never;

pub use never::Never;

