use chrono::{DateTime, Local};
use ics::properties::{Description, DtEnd, DtStart, Location, Summary};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
	summary: Option<String>,
	dt_start: Option<DateTime<Local>>,
	dt_end: Option<DateTime<Local>>,
	all_day: bool,
	location: Option<String>,
	description: Option<String>,
}

impl Event {
	/// uses a randomized uuid
	pub fn new() -> Self {
		Self {
			summary: None,
			dt_start: None,
			dt_end: None,
			all_day: false,
			location: None,
			description: None,
		}
	}

	// ---

	pub fn summary(&mut self, summary: impl Into<String>) -> &mut Self {
		self.summary = Some(summary.into());
		self
	}

	// pub fn dt_start_str(&mut self, dt_start: impl Into<String>) -> &mut Self {
	// 	self.dt_start = Some(dt_start.into());
	// 	self
	// }
	// pub fn dt_end_str(&mut self, dt_end: impl Into<String>) -> &mut Self {
	// 	self.dt_end = Some(dt_end.into());
	// 	self
	// }
	pub fn location(&mut self, location: impl Into<String>) -> &mut Self {
		self.location = Some(location.into());
		self
	}

	pub fn dt_start(&mut self, dt_start: chrono::DateTime<Local>) -> &mut Self {
		self.dt_start = Some(dt_start);
		// self.dt_start_str(dt_start.format("%Y%m%dT%H%M%S").to_string())
		self
	}
	pub fn get_dt_start(&self) -> Option<&chrono::DateTime<Local>> {
		self.dt_start.as_ref()
	}
	pub fn dt_end(&mut self, dt_end: chrono::DateTime<Local>) -> &mut Self {
		// self.dt_end_str(dt_end.format("%Y%m%dT%H%M%S").to_string())
		self.dt_end = Some(dt_end);
		self
	}
	pub fn get_dt_end(&self) -> Option<&chrono::DateTime<Local>> {
		self.dt_end.as_ref()
	}

	/// the event is all day on a single day \
	/// it shouldn't matter what time it is only the date
	pub fn dt_all_day(&mut self, dt_day: chrono::DateTime<Local>) -> &mut Self {
		// let day = dt_day.format("%Y%m%d").to_string();
		// self.dt_start_str(day.clone());
		// self.dt_end_str(day);
		self.dt_start(dt_day);
		self.dt_end(dt_day);
		self.all_day = true;

		self
	}

	/// replaces the old description
	pub fn description_replace<'a>(&mut self, description: impl Into<String>) -> &mut Self {
		self.description = Some(description.into());
		self
	}
	/// appends to the old description (if it exists) \
	/// does not put a new line anywhere just straight appends
	pub fn description(&mut self, description: impl AsRef<str>) -> &mut Self {
		let description = description.as_ref();
		match &mut self.description {
			Some(existing) => {
				existing.push_str(description);
			}
			None => self.description = Some(description.to_string()),
		}
		self
	}
}
impl Event {
	/// formats a basedtime based on whether the event's all day or not
	fn format_datetime(&self, datetime: DateTime<Local>) -> String {
		if !self.all_day {
			datetime.format("%Y%m%dT%H%M%S").to_string()
		} else {
			datetime.format("%Y%m%d").to_string()
		}
	}

	/// returns none if there's no start date
	pub fn to_ics_event(self) -> Option<ics::Event<'static>> {
		// let date_day = exam.date.format("%Y%m%d").to_string();
		// igy kell formazni a masodik argot sztem a start timebol mehet

		let dt_start = match self.dt_start {
			Some(a) => a,
			None => return None,
		};
		let dt_start_str = self.format_datetime(dt_start);
		let dt_end_str = self.dt_end.map(|dt_end| self.format_datetime(dt_end));

		let uuid = uuid::Uuid::new_v4();
		let uuid = format!("{uuid}");
		let mut ics_event: ics::Event<'static> = ics::Event::new(uuid, dt_start_str.clone());

		match self.summary {
			Some(a) => ics_event.push(Summary::new(a)),
			None => {}
		}
		ics_event.push(DtStart::new(dt_start_str));
		match dt_end_str {
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
