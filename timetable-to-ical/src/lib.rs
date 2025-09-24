use std::borrow::Cow;

use ics::{
	Event, ICalendar,
	properties::{Comment, Description, DtEnd, DtStart, Location, Summary},
};
use kreta_rs::client::timetable::LessonRaw;

pub mod absence_best_guess;
use absence_best_guess::absence_guess;

pub mod err;

use crate::absence_best_guess::Absence;

#[derive(Clone, Debug)]
pub struct Options {
	pub lowercase_subject_names: bool,
	pub lesson_topic_in_name: bool,
	pub teacher_name_in_location: bool,

	pub substitution_prefix: Cow<'static, str>,
	pub announced_exam_prefix: Cow<'static, str>,
	pub absence_prefix: Cow<'static, str>,
	pub student_late_prefix: Cow<'static, str>,

	/// includes a pretty print (basic rust {:#?}) of the entire [LessonRaw] as notes
	pub pretty_print_as_desc: bool,
}
impl Default for Options {
	fn default() -> Self {
		Self {
			lowercase_subject_names: true,
			lesson_topic_in_name: true,
			teacher_name_in_location: true,
			pretty_print_as_desc: false,
			substitution_prefix: "🔄".into(),
			announced_exam_prefix: "📝".into(),
			absence_prefix: "🚫".into(),
			student_late_prefix: "⏰".into(),
		}
	}
}

pub fn map_lessons_to_events<'a, I: IntoIterator<Item = &'a LessonRaw>>(
	iter: I,
	opts: &Options,
) -> impl Iterator<Item = Event<'a>> {
	iter.into_iter().map(|lesson| {
		let uid = uuid::Uuid::new_v4();
		let uid = format!("{uid}");

		let start_escaped = lesson.start_time.replace('-', "").replace(':', "");
		let end_escaped = lesson.end_time.replace('-', "").replace(':', "");

		let name = {
			let name_base: Cow<'a, str> = if opts.lowercase_subject_names {
				lesson.name.to_lowercase().into()
			} else {
				(&lesson.name).into()
			};

			let mut name_prefixes = String::new();
			let absence = absence_guess(&lesson.student_presence.name);
			match absence {
				Absence::Absent => {
					if opts.absence_prefix.len() > 0 {
						name_prefixes.push_str(&opts.absence_prefix);
					}
				}
				Absence::Late => {
					if opts.student_late_prefix.len() > 0 {
						name_prefixes.push_str(&opts.student_late_prefix);
					}
				}
				_ => {}
			}
			if lesson.announced_exam_uid.is_some() && opts.announced_exam_prefix.len() > 0 {
				name_prefixes.push_str(&opts.announced_exam_prefix);
			}
			if lesson.substitute_teacher_name.is_some() && opts.substitution_prefix.len() > 0 {
				name_prefixes.push_str(&opts.substitution_prefix);
			}

			let name_suffixes = if lesson.topic.is_some() && opts.lesson_topic_in_name {
				let topic = lesson
					.topic
					.as_ref()
					.expect("we just checked that topic.is_some() == true");
				format!(" - {topic}")
			} else {
				String::new()
			};

			match (name_prefixes.len(), name_suffixes.len()) {
				(0, 0) => name_base,
				(p, _) => {
					if p > 0 {
						name_prefixes.push(' ');
					}

					format!("{name_prefixes}{name_base}{name_suffixes}").into()
				}
			}
		};

		let location: Cow<'a, str> = {
			let room_name = &lesson.room_name;
			if opts.teacher_name_in_location {
				let teachers_name = match &lesson.substitute_teacher_name {
					Some(a) => a,
					None => &lesson.teachers_name,
				};
				format!("{room_name} - {teachers_name}").into()
			} else {
				room_name.into()
			}
		};

		let mut event = Event::new(uid, &lesson.start_time);
		event.push(Summary::new(name));
		event.push(DtStart::new(start_escaped));
		event.push(DtEnd::new(end_escaped));
		event.push(Location::new(location));

		if opts.pretty_print_as_desc {
			let desc = format!("{lesson:#?}");
			let desc = escape_desc_text(&desc);
			event.push(Description::new(desc));
		}

		event
	})
}

pub fn lessons_to_calendar<'a, I: IntoIterator<Item = &'a LessonRaw>>(
	iter: I,
	opts: &Options,
) -> ICalendar<'a> {
	let mut calendar = ICalendar::new("2.0", "timetable-to-ical");

	let events_iter = map_lessons_to_events(iter, opts);
	for event in events_iter {
		calendar.add_event(event);
	}

	calendar
}

pub fn lessons_to_calendar_file<'a, I: IntoIterator<Item = &'a LessonRaw>>(
	iter: I,
	opts: &Options,
) -> String {
	let calendar = lessons_to_calendar(iter, opts);
	calendar.to_string()
}

// -- utils

fn escape_desc_text(input: &str) -> String {
	input
		.replace('\\', "\\\\")
		.replace(';', "\\;")
		.replace(',', "\\,")
		.replace('\n', "\\n")
}
