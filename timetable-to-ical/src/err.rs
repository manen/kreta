use std::fmt::{Debug, Display};

use calhelp::{Calendar, Event};
use chrono::Local;

/// T is a calendar. E is an error. if the result is an error, turn it into a calendar.
pub fn result_as_timetable<E: Display + Debug>(res: Result<String, E>) -> String {
	match res {
		Ok(a) => a,
		Err(err) => {
			let today = Local::now();

			let mut err_event = Event::new();
			err_event.summary("timetable error");
			err_event.dt_all_day(today);
			err_event.location("see event notes for details");

			let notes = format!("{err}\n\n{err:?}");
			err_event.description(&notes);

			let calendar = Calendar::new(std::iter::once(err_event));
			calendar.to_string_publish()
		}
	}
}

pub fn handle_timetable_err<E: Display + Debug, F: FnOnce() -> Result<String, E>>(f: F) -> String {
	let res = f();
	result_as_timetable(res)
}

pub async fn handle_timetable_err_async<
	E: Display + Debug,
	F: Future<Output = Result<String, E>>,
>(
	future: F,
) -> String {
	let res = future.await;
	result_as_timetable(res)
}
