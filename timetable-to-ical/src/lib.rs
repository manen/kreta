use ics::{
	Event, ICalendar,
	properties::{Comment, DtEnd, DtStart, Location, Summary},
};
use kreta_rs::client::timetable::LessonRaw;

pub fn map_lessons_to_events<'a, I: IntoIterator<Item = &'a LessonRaw>>(
	iter: I,
) -> impl Iterator<Item = Event<'a>> {
	iter.into_iter().map(|lesson| {
		let uid = uuid::Uuid::new_v4();
		let uid = format!("{uid}");

		let start_escaped = lesson.start_time.replace('-', "").replace(':', "");
		let end_escaped = lesson.end_time.replace('-', "").replace(':', "");

		let mut event = Event::new(uid, &lesson.start_time);
		event.push(Summary::new(&lesson.name));
		// event.push(Comment::new(format!("{lesson:#?}")));
		event.push(DtStart::new(start_escaped));
		event.push(DtEnd::new(end_escaped));
		event.push(Location::new(&lesson.room_name));

		event
	})
}

pub fn lessons_to_calendar<'a, I: IntoIterator<Item = &'a LessonRaw>>(iter: I) -> ICalendar<'a> {
	let mut calendar = ICalendar::new("2.0", "timetable-to-ical");

	let events_iter = map_lessons_to_events(iter);
	for event in events_iter {
		calendar.add_event(event);
	}

	calendar
}

pub fn lessons_to_calendar_file<'a, I: IntoIterator<Item = &'a LessonRaw>>(iter: I) -> String {
	let calendar = lessons_to_calendar(iter);
	calendar.to_string()
}
