use chrono::Local;
use ics::properties::{Description, DtEnd, DtStart, Location, Summary};

#[derive(Clone, Debug)]
pub struct Event {
	summary: Option<String>,
	dt_start: Option<String>,
	dt_end: Option<String>,
	location: Option<String>,
	description: Option<String>,
}

impl Event {
	/// uses a randomized uuid
	pub fn new() -> Self {
		Self {
			summary: None,
			dt_start: None,
			dt_end: None, // todo: timing enum
			location: None,
			description: None,
		}
	}

	// ---

	pub fn summary(&mut self, summary: impl Into<String>) -> &mut Self {
		self.summary = Some(summary.into());
		self
	}

	pub fn dt_start_str(&mut self, dt_start: impl Into<String>) -> &mut Self {
		self.dt_start = Some(dt_start.into());
		self
	}
	pub fn dt_end_str(&mut self, dt_end: impl Into<String>) -> &mut Self {
		self.dt_end = Some(dt_end.into());
		self
	}
	pub fn location(&mut self, location: impl Into<String>) -> &mut Self {
		self.location = Some(location.into());
		self
	}

	pub fn dt_start(&mut self, dt_start: chrono::DateTime<Local>) -> &mut Self {
		self.dt_start_str(dt_start.format("%Y%m%dT%H%M%S").to_string())
	}
	pub fn dt_end(&mut self, dt_end: chrono::DateTime<Local>) -> &mut Self {
		self.dt_end_str(dt_end.format("%Y%m%dT%H%M%S").to_string())
	}

	/// the event is all day on a single day
	pub fn dt_all_day(&mut self, dt_day: chrono::NaiveDate) -> &mut Self {
		let day = dt_day.format("%Y%m%d").to_string();
		self.dt_start_str(day.clone());
		self.dt_end_str(day);
		self
	}

	/// rename this to description_new and have description append to the end if there's already a desc
	pub fn description<'a>(&mut self, description: impl Into<String>) -> &mut Self {
		self.description = Some(description.into());
		self
	}

	/// returns none if there's no start date
	pub fn to_ics_event(self) -> Option<ics::Event<'static>> {
		// let date_day = exam.date.format("%Y%m%d").to_string();
		// igy kell formazni a masodik argot sztem a start timebol mehet

		let dt_stamp = match self.dt_start {
			Some(a) => a,
			None => return None,
		};

		let uuid = uuid::Uuid::new_v4();
		let uuid = format!("{uuid}");
		let mut ics_event: ics::Event<'static> = ics::Event::new(uuid, dt_stamp.clone());

		match self.summary {
			Some(a) => ics_event.push(Summary::new(a)),
			None => {}
		}
		ics_event.push(DtStart::new(dt_stamp));
		match self.dt_end {
			Some(a) => ics_event.push(DtEnd::new(a)),
			None => {}
		}
		match self.location {
			Some(a) => ics_event.push(Location::new(a)),
			None => {}
		};

		fn escape_desc_text(input: &str) -> String {
			input
				.replace('\\', "\\\\")
				.replace(';', "\\;")
				.replace(',', "\\,")
				.replace('\n', "\\n")
		}

		match self.description {
			Some(desc) => {
				let desc_escaped = escape_desc_text(&desc);
				ics_event.push(Description::new(desc_escaped))
			}
			None => {}
		}

		Some(ics_event)
	}
}
