use std::borrow::Cow;

use anyhow::Context;
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Budapest;
use ics::{
	Event, ICalendar,
	properties::{Description, DtEnd, DtStart, Location, Summary},
};
use kreta_rs::client::{exam::ExamRaw, homework::HomeworkRaw, timetable::LessonRaw};

pub mod absence_best_guess;
use absence_best_guess::absence_guess;

#[cfg(feature = "combine")]
pub mod combine;

pub mod err;

use crate::absence_best_guess::Absence;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Options {
	pub lowercase_subject_names: bool,
	pub lesson_topic_in_name: bool,
	pub teacher_name_in_location: bool,

	// class info
	pub substitution_prefix: Cow<'static, str>,
	/// elmarado ora
	pub cancelled_lesson_preifx: Cow<'static, str>,

	pub announced_exam_prefix: Cow<'static, str>,
	/// homework prefix appears on the day the homework is attached to, not the deadline lesson
	pub homework_given_prefix: Cow<'static, str>,

	pub absence_prefix: Cow<'static, str>,
	pub student_late_prefix: Cow<'static, str>,
	// class info end
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
			cancelled_lesson_preifx: "❌".into(),
			announced_exam_prefix: "📝".into(),
			homework_given_prefix: "🏠".into(),
			absence_prefix: "🚫".into(),
			student_late_prefix: "⏰".into(),
		}
	}
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ExtraData<'a> {
	/// false means show homework symbol where it's attached to the lesson (so it'll show up on the lesson it was given)
	/// true means it'll show up when it's due (if it's attached in this struct)
	is_homework_included: bool,

	homework: Option<&'a HomeworkRaw>,
	exam: Option<&'a ExamRaw>,
}

const FORMAT_DATE: &str = "%Y%m%d";
const FORMAT_DATETIME: &str = "%Y%m%dT%H%M%SZ";

/// avoid throwing errors if possible
pub fn lesson_to_event_explicit<'a>(
	lesson: &'a LessonRaw,
	opts: &Options,
	extra_data: ExtraData<'a>,
) -> anyhow::Result<Event<'a>> {
	let uid = uuid::Uuid::new_v4();
	let uid = format!("{uid}");

	let (dtstamp, start_escaped, end_escaped) = {
		let start: DateTime<Utc> = lesson
			.start_time
			.parse()
			.with_context(|| format!("while parsing {} as a datetime", lesson.start_time))?;
		let end: DateTime<Utc> = lesson
			.end_time
			.parse()
			.with_context(|| format!("while parsing {} as a datetime", lesson.end_time))?;
		// let (start, end) = (start.with_timezone(&Budapest), end.with_timezone(&Budapest));

		if start == end {
			// if start_time == end_time => make it an all day event (so far only seems to be school holidays n shit)
			let start = start.with_timezone(&Budapest);
			let event_time = start.format(FORMAT_DATE).to_string();
			(
				start.format("%Y%m%dT000000Z").to_string(),
				event_time.clone(),
				event_time,
			)
		} else {
			let dtstamp = start.format("%Y%m%dT000000Z").to_string();
			let start = start.format(FORMAT_DATETIME).to_string();
			let end = end.format(FORMAT_DATETIME).to_string();

			(dtstamp, start, end)
		}
	};

	let name = {
		let name_base: Cow<str> = if opts.lowercase_subject_names {
			lesson.name.to_lowercase().into()
		} else {
			(&lesson.name).into()
		};

		let mut name_prefixes = String::new();
		let absence = lesson
			.student_presence
			.as_ref()
			.map(|a| absence_guess(&a.name))
			.unwrap_or(Absence::Present);
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
		if lesson.status.uid.contains("Elmaradt") && opts.cancelled_lesson_preifx.len() > 0 {
			name_prefixes.push_str(&opts.cancelled_lesson_preifx);
		}
		if lesson.announced_exam_uid.is_some() && opts.announced_exam_prefix.len() > 0 {
			name_prefixes.push_str(&opts.announced_exam_prefix);
		}
		let show_homework = match extra_data.is_homework_included {
			false => lesson.homework_uid.is_some(),
			true => extra_data.homework.is_some(),
		};
		if show_homework && opts.homework_given_prefix.len() > 0 {
			name_prefixes.push_str(&opts.homework_given_prefix);
		}
		if lesson.substitute_teacher_name.is_some() && opts.substitution_prefix.len() > 0 {
			name_prefixes.push_str(&opts.substitution_prefix);
		}

		let topic = match &lesson.topic {
			Some(topic) => Some(topic),
			None => match extra_data.exam {
				Some(exam) => Some(&exam.topic),
				None => None,
			},
		};

		let name_suffixes = if topic.is_some() && opts.lesson_topic_in_name {
			let topic = topic.expect("we just checked that topic.is_some() == true");
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

	let location: Cow<str> = {
		let room_name = lesson.room_name.as_ref();
		let teachers_name = if opts.teacher_name_in_location {
			let teachers_name: Option<&String> = match &lesson.substitute_teacher_name {
				Some(a) => Some(a),
				None => lesson.teachers_name.as_ref().map(|a| a.into()),
			};
			teachers_name
		} else {
			None
		};

		match (room_name, teachers_name) {
			(Some(room), Some(teacher)) => format!("{room} - {teacher}").into(),
			(Some(room), None) => room.into(),
			(None, Some(teacher)) => teacher.into(),
			(None, None) => "".into(),
		}
	};

	let mut event = Event::new(uid, dtstamp);
	event.push(Summary::new(name));
	event.push(DtStart::new(start_escaped));
	event.push(DtEnd::new(end_escaped));
	event.push(Location::new(location));

	let pretty_print = if opts.pretty_print_as_desc {
		let desc = format!("{lesson:#?}\n\n{extra_data:#?}");
		Some(desc)
	} else {
		None
	};

	let mut info = String::new();
	match extra_data.exam {
		Some(exam) => {
			info += &format!(
				"{} {}\n{}",
				opts.announced_exam_prefix, exam.topic, exam.method.desc
			);
		}
		None => {}
	}
	match extra_data.homework {
		Some(hw) => {
			let date_assigned: DateTime<Utc> = hw.date_assigned.parse().with_context(|| {
				format!(
					"while parsing homework.date_assigned as DateTime: {}",
					hw.date_assigned
				)
			})?;
			let date_assigned = date_assigned.with_timezone(&Budapest);
			let date_assigned = date_assigned.format("%Y %B %d %H:%M:%S");
			info += &format!(
				"{}\n{}\n - {}, {date_assigned}",
				opts.homework_given_prefix, hw.text, hw.teachers_name
			);
		}
		None => {}
	}

	let desc = match (pretty_print, info.len()) {
		(Some(pretty_print), 0) => Some(pretty_print),
		(Some(pretty_print), _) => Some(format!("{info}\n\n{pretty_print}")),
		(None, 0) => None,
		(None, _) => Some(info),
	};
	if let Some(desc) = desc {
		let desc = escape_desc_text(&desc);
		event.push(Description::new(desc));
	}

	Ok(event)
}

pub fn lesson_to_event<'a>(lesson: &'a LessonRaw, opts: &Options) -> anyhow::Result<Event<'a>> {
	lesson_to_event_explicit(lesson, opts, Default::default())
}

pub fn lessons_to_calendar_file_res<'a, I: IntoIterator<Item = &'a LessonRaw>>(
	iter: I,
	opts: &Options,
) -> anyhow::Result<String> {
	let mut calendar = ICalendar::new("2.0", "timetable-to-ical");

	let events_iter = iter
		.into_iter()
		.map(|lesson| (lesson, lesson_to_event(lesson, opts)));
	for (lesson, event) in events_iter {
		let event = event.with_context(|| {
			format!("calling lesson_to_event returned an error\nlesson:\n{lesson:#?}")
		})?;

		calendar.add_event(event);
	}

	Ok(calendar.to_string())
}
/// errors get turned into a timetable
pub fn lessons_to_calendar_file<'a, I: IntoIterator<Item = &'a LessonRaw>>(
	iter: I,
	opts: &Options,
) -> String {
	err::result_as_timetable(lessons_to_calendar_file_res(iter, opts))
}

// -- utils

fn escape_desc_text(input: &str) -> String {
	input
		.replace('\\', "\\\\")
		.replace(';', "\\;")
		.replace(',', "\\,")
		.replace('\n', "\\n")
}
