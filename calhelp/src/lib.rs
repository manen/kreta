//! crate to help with making importable calendars
//!
//! currently timetable-to-ical returns a single string so once events are added you can't add anything else \
//! this isn't flexible so:
//! now there's two middleground types between a concept and a calendar as a string and it's Event and Calendar and both are modifyable and mergable so it's all nice and flexible

mod event;
pub use event::*;

use ics::{ICalendar, properties::Method};
use serde::{Deserialize, Serialize};

/// owns everything
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Calendar {
	events: Vec<Event>,
}
impl Calendar {
	pub fn new<I: IntoIterator<Item = Event>>(iter: I) -> Self {
		Self {
			events: iter.into_iter().collect(),
		}
	}

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

	pub fn to_string_publish(&self) -> String {
		let mut calendar = ICalendar::new("2.0", "timetable-to-ical");
		calendar.push(Method::new("PUBLISH"));

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
