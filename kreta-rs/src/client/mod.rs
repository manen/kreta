#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;

pub mod refresh;

pub mod homework;
pub mod timetable;
