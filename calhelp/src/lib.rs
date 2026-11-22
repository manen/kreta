//! crate to help with making importable calendars
//!
//! currently timetable-to-ical returns a single string so once events are added you can't add anything else \
//! this isn't flexible so:

mod event;
pub use event::*;

use ics::ICalendar;
use serde::{Deserialize, Serialize};

/// owns everything
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Calendar {
	events: Vec<Event>,
}
impl Calendar {
	pub fn new<I: IntoIterator<Item = Event>>(iter: I) -> Self {
		Self {
			events: iter.into_iter().collect(),
		}
	}

	// todo method: publish

	pub fn merge(self, other: Self) -> Self {
		Calendar {
			events: self
				.events
				.into_iter()
				.chain(other.events.into_iter())
				.collect(),
		}
	}

	pub fn events(&self) -> impl Iterator<Item = &Event> {
		self.events.iter()
	}
	pub fn take_events(self) -> impl Iterator<Item = Event> {
		self.events.into_iter()
	}

	pub fn to_string(&self) -> String {
		let mut calendar = ICalendar::new("2.0", "timetable-to-ical");

		for event in self.events.iter().cloned() {
			match event.to_ics_event() {
				Some(event) => calendar.add_event(event),
				None => {}
			}
		}

		calendar.to_string()
	}
}

impl FromIterator<Event> for Calendar {
	fn from_iter<T: IntoIterator<Item = Event>>(iter: T) -> Self {
		Self::new(iter)
	}
}
