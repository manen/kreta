use std::borrow::Cow;

use ics::{
	Event, ICalendar,
	properties::{DtEnd, DtStart, Location, Summary},
};
use kreta_rs::client::timetable::LessonRaw;

#[derive(Clone, Debug)]
pub struct Options {
	pub lowercase_subject_names: bool,
	pub substitution_prefix: Cow<'static, str>,
	pub announced_exam_prefix: Cow<'static, str>,
}
impl Default for Options {
	fn default() -> Self {
		Self {
			lowercase_subject_names: true,
			substitution_prefix: "🔄".into(),
			announced_exam_prefix: "📝".into(),
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
			if lesson.announced_exam_uid.is_some() && opts.announced_exam_prefix.len() > 0 {
				name_prefixes.push_str(&opts.announced_exam_prefix);
			}
			if lesson.substitute_teacher_name.is_some() && opts.substitution_prefix.len() > 0 {
				name_prefixes.push_str(&opts.substitution_prefix);
			}

			match name_prefixes.len() {
				0 => name_base,
				1.. => {
					name_prefixes.push(' ');
					format!("{name_prefixes}{name_base}").into()
				}
			}
		};

		let mut event = Event::new(uid, &lesson.start_time);
		event.push(Summary::new(name));
		// event.push(Comment::new(format!("{lesson:#?}")));
		event.push(DtStart::new(start_escaped));
		event.push(DtEnd::new(end_escaped));
		event.push(Location::new(&lesson.room_name));

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
