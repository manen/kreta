#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;

pub mod refresh;

pub mod exam;
pub mod homework;
pub mod timetable;
pub mod absences;
