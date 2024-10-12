#![feature(thread_local)]
#![feature(once_cell_try)]
#![deny(elided_lifetimes_in_paths)]

mod error;
pub mod platform;
pub mod traits;

pub use error::*;
