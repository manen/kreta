use ics::ICalendar;
use kreta_rs::client::Client;

use crate::Options;

pub async fn combined_calendar_file(
	client: &Client,
	from: &str,
	to: &str,
	opts: &Options,
) -> anyhow::Result<String> {
	let combined = kreta_combine::get_combined(client, from, to).await?;

	let events = combined.iter().map(|lesson| {
		let extra_data = crate::ExtraData {
			is_homework_included: true,
			homework: lesson.homework.as_ref(),
			exam: lesson.exam.as_ref(),
		};
		crate::lesson_to_event_explicit(&lesson.lesson_raw, opts, extra_data)
	});
	let calendar = {
		let mut calendar = ICalendar::new("2.0", "timetable-to-ical");
		for event in events {
			calendar.add_event(event);
		}

		calendar.to_string()
	};

	Ok(calendar)
}
