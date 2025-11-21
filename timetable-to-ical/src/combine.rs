use anyhow::Context;
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Budapest;
use ics::{
	Event, ICalendar,
	properties::{Description, DtEnd, DtStart, Location, Summary},
};
use kreta_combine::CombinedLesson;
use kreta_rs::client::Client;

use crate::Options;

pub fn map_combined<'a>(
	combined: impl IntoIterator<Item = &'a CombinedLesson>,
	opts: &Options,
) -> impl Iterator<Item = anyhow::Result<Event<'a>>> {
	let events = combined.into_iter().map(|lesson| {
		let extra_data = crate::ExtraData {
			is_homework_included: true,
			homework: lesson.homework.as_ref(),
			exam: lesson.exam.as_ref(),
		};

		crate::lesson_to_event_explicit(&lesson.lesson_raw, opts, extra_data)
			.with_context(|| format!("error while turning lesson into event\n{lesson:#?}"))
	});
	events
}

pub fn collect_from_combined<'a>(
	iter: impl IntoIterator<Item = anyhow::Result<Event<'a>>>,
) -> anyhow::Result<String> {
	let mut calendar = ICalendar::new("2.0", "timetable-to-ical");
	for event in iter {
		let event = event?;
		calendar.add_event(event);
	}

	Ok(calendar.to_string())
}

pub async fn combined_range_calendar_file(
	client: &Client,
	from: chrono::DateTime<chrono::Utc>,
	to: chrono::DateTime<chrono::Utc>,
	opts: &Options,
) -> anyhow::Result<String> {
	let preprocessed = kreta_combine::get_preprocessed_range(client, from, to)
		.await
		.with_context(|| "while calling kreta_combine::get_preprocessed_range")?;

	let (combined, remaining_homework, remaining_exams) =
		kreta_combine::match_preprocessed_with_remainder(preprocessed)?;

	let iter = map_combined(combined.iter(), opts);

	// -- add remaining entries
	let remaining_homework_iter = remaining_homework.into_iter().map(|(_, homework)| {
		let uid = uuid::Uuid::new_v4();
		let uid = format!("{uid}");

		let deadline: DateTime<Utc> = homework.date_deadline.parse().with_context(|| {
			format!(
				"while parsing homework deadline {} as a datetime",
				homework.date_deadline
			)
		})?;
		let deadline_day = deadline.with_timezone(&Budapest);
		let deadline_day = deadline_day.format("%Y%m%d").to_string();

		let mut event = Event::new(uid, deadline_day.clone());
		event.push(Summary::new(format!(
			"{} {}",
			opts.homework_given_prefix, homework.subject_name
		)));
		event.push(DtStart::new(deadline_day.clone()));
		event.push(DtEnd::new(deadline_day));

		let date_assigned: DateTime<Utc> = homework.date_assigned.parse().with_context(|| {
			format!(
				"while parsing homework.date_assigned as DateTime: {}",
				homework.date_assigned
			)
		})?;
		let date_assigned = date_assigned.with_timezone(&Budapest);
		let date_assigned = date_assigned.format("%Y %B %d %H:%M:%S");
		let desc = format!(
			"{}\n{}\n - {}, {date_assigned}",
			opts.homework_given_prefix, homework.text, homework.teachers_name
		);
		event.push(Description::new(crate::escape_desc_text(&desc)));

		anyhow::Ok(event)
	});

	let remaining_exams_iter = remaining_exams.into_iter().map(|(_, exam)| {
		let uid = uuid::Uuid::new_v4();
		let uid = format!("{uid}");

		let date: DateTime<Utc> = exam
			.date
			.parse()
			.with_context(|| format!("while parsing exam date {} as a datetime", exam.date))?;
		let date = date.with_timezone(&Budapest);
		let date_day = date.format("%Y%m%d").to_string();

		let mut event = Event::new(uid, date_day.clone());
		event.push(Summary::new(format!(
			"{} {} - {}",
			opts.announced_exam_prefix, exam.subject_name, exam.topic
		)));
		event.push(DtStart::new(date_day.clone()));
		event.push(DtEnd::new(date_day.clone()));
		event.push(Location::new(exam.method.desc.clone()));

		let desc = format!("{:#?}", exam);
		event.push(Description::new(crate::escape_desc_text(&desc)));

		anyhow::Ok(event)
	});

	let iter_with_remainder = iter
		.chain(remaining_homework_iter)
		.chain(remaining_exams_iter);

	let timetable = collect_from_combined(iter_with_remainder)?;
	Ok(timetable)
}
