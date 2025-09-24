use std::fmt::{Debug, Display};

use chrono::Utc;
use ics::{
	Event, ICalendar,
	properties::{Description, DtEnd, DtStart, Location, Summary},
};

/// T is a calendar. E is an error. if the result is an error, turn it into a calendar.
pub fn result_as_timetable<E: Display + Debug>(res: Result<String, E>) -> String {
	match res {
		Ok(a) => a,
		Err(err) => {
			let today = Utc::now().date_naive();
			let today = today.format("%Y%m%d").to_string();

			let uuid = uuid::Uuid::new_v4();
			let uuid = format!("{uuid}");

			let mut err_event = Event::new(&uuid, &today);
			err_event.push(Summary::new("timetable error"));
			err_event.push(DtStart::new(&today));
			err_event.push(DtEnd::new(&today));
			err_event.push(Location::new("see event notes for details"));

			let notes = format!("{err}\n\n{err:#?}");
			err_event.push(Description::new(super::escape_desc_text(&notes)));

			let mut calendar = ICalendar::new("2.0", "timetable-to-ical");
			calendar.add_event(err_event);

			calendar.to_string()
		}
	}
}
