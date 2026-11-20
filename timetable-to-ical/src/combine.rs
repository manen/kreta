use anyhow::Context;
use calhelp::{Calendar, Event};
use chrono_tz::Europe::Budapest;
use kreta_combine::CombinedLesson;
use kreta_rs::client::Client;

use crate::Options;

pub fn map_combined<'a>(
	combined: impl IntoIterator<Item = &'a CombinedLesson>,
	opts: &Options,
) -> impl Iterator<Item = anyhow::Result<Event>> {
	let events = combined.into_iter().map(|lesson| {
		let extra_data = crate::ExtraData {
			is_homework_included: true,
			homework: lesson.homework.as_ref(),
			exam: lesson.exam.as_ref(),
			absence: lesson.absence.as_ref(),
		};

		crate::lesson_to_event_explicit(&lesson.lesson_raw, opts, extra_data)
			.with_context(|| format!("error while turning lesson into event\n{lesson:#?}"))
	});
	events
}

// pub fn collect_from_combined<'a>(
// 	iter: impl IntoIterator<Item = anyhow::Result<Event<'static>>>,
// ) -> anyhow::Result<String> {
// 	let mut calendar = ICalendar::new("2.0", "timetable-to-ical");
// 	for event in iter {
// 		let event = event?;
// 		calendar.add_event(event);
// 	}
//
// 	Ok(calendar.to_string())
// }

pub async fn combined_range_calendar_file(
	client: &Client,
	from: chrono::DateTime<chrono::Local>,
	to: chrono::DateTime<chrono::Local>,
	opts: &Options,
) -> anyhow::Result<String> {
	combined_range_calendar(client, from, to, opts)
		.await
		.map(|cal| cal.to_string())
}

pub async fn combined_range_calendar(
	client: &Client,
	from: chrono::DateTime<chrono::Local>,
	to: chrono::DateTime<chrono::Local>,
	opts: &Options,
) -> anyhow::Result<Calendar> {
	let (combined, remaining_homework, remaining_exams) =
		// kreta_combine::match_preprocessed_with_remainder(preprocessed)?;
		kreta_combine::fetch_combined_with_remainder(client, from, to).await?;

	let iter = map_combined(combined.iter(), opts);

	// -- add remaining entries
	let remaining_homework_iter = remaining_homework.into_iter().map(|(_, homework)| {
		let uid = uuid::Uuid::new_v4();
		let uid = format!("{uid}");

		let deadline_day = homework.date_deadline.with_timezone(&Budapest);

		// let mut event = Event::new(uid, deadline_day.clone());
		// event.push(Summary::new(format!(
		// 	"{} {}",
		// 	opts.homework_given_prefix, homework.subject_name
		// )));
		// event.push(DtStart::new(deadline_day.clone()));
		// event.push(DtEnd::new(deadline_day));
		//

		let mut event = Event::new();
		event
			.summary(format!(
				"{} {}",
				opts.homework_given_prefix, homework.subject_name
			))
			.dt_all_day(deadline_day.date_naive());

		let date_assigned = homework.date_assigned.format("%Y %B %d");

		let pretty_print = if opts.pretty_print_as_desc {
			format_args!("\n\n{homework:#?}")
		} else {
			format_args!("")
		};
		let desc = format!(
			"{}\n{}\n - {}, {date_assigned}{pretty_print}",
			opts.homework_given_prefix,
			homework.text_extract(),
			homework.teachers_name
		);
		// event.description(crate::escape_desc_text(&desc));
		event.description(&desc);

		anyhow::Ok(event)
	});

	let remaining_exams_iter = remaining_exams.into_iter().map(|(_, exam)| {
		let mut event = Event::new();

		let pretty_print = if opts.pretty_print_as_desc {
			format_args!("\n\n{exam:#?}")
		} else {
			format_args!("")
		};

		// 		event.push(Summary::new(format!(
		// 			"{} {} - {}{pretty_print}",
		// 			opts.announced_exam_prefix, exam.subject_name, exam.topic
		// 		)));
		// 		event.push(DtStart::new(date_day.clone()));
		// 		event.push(DtEnd::new(date_day.clone()));
		// 		event.push(Location::new(exam.method.desc.clone()));
		//
		// 		let desc = format!("{:#?}", exam);
		// 		event.push(Description::new(crate::escape_desc_text(&desc)));

		event
			.summary(format!(
				"{} {} - {}{pretty_print}",
				opts.announced_exam_prefix, exam.subject_name, exam.topic
			))
			.dt_all_day(exam.date.date_naive())
			.location(exam.method.desc.clone());

		anyhow::Ok(event)
	});

	let iter_with_remainder = iter
		.chain(remaining_homework_iter)
		.chain(remaining_exams_iter)
		// .chain(std::iter::once({
		// 	let mut event = Event::new();
		// 	event.summary("fasz");
		// 	event.dt_all_day(chrono::Local::now().date_naive());
		// 	Ok(event)
		// }))
	;

	let events = iter_with_remainder.collect::<anyhow::Result<Calendar>>()?;
	Ok(events)
}
