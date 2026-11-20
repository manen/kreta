use std::{collections::HashMap, fmt::Display};

use crate::AbsenceRaw;
use anyhow::Context;
use chrono::{Datelike, Duration, Local};

use crate::{AbsencesByExcuse, retreive::last_september_first_expl};

/// denotes which week a date is in from september 1st
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WeekNum(u32);
impl WeekNum {
	pub fn from_date(date: chrono::DateTime<Local>) -> Self {
		let sept_1 = last_september_first_expl(date.clone());

		let date_monday = date - Duration::days(date.weekday().num_days_from_monday() as _);
		let sept_1_monday = sept_1 - Duration::days(sept_1.weekday().num_days_from_monday() as _);

		let diff_days = (date_monday - sept_1_monday).num_days();
		let diff_weeks_i64 = diff_days / 7;

		let diff_weeks = match diff_weeks_i64.try_into() {
			Ok(a) => {
				let a: u32 = a;
				a
			}
			Err(err) => {
				let clamped = diff_weeks_i64.clamp(0, u32::MAX as i64) as u32;
				eprintln!(
					"failed to turn weeknum from i64 to u32, clamping from {diff_weeks_i64} to {clamped} for date {date:?}\n{err}"
				);
				clamped
			}
		};

		Self(diff_weeks)
	}
	// 	pub fn get_monday_expl(self, sept_1: DateTime<Local>) -> chrono::DateTime<Local> {
	// 		let sept_1_monday = sept_1 - Duration::days(sept_1.weekday().num_days_from_monday() as _);
	// 		let date_monday = sept_1_monday + Duration::weeks(self.0 as _);
	//
	// 		date_monday
	// 	}

	pub fn take(self) -> u32 {
		self.0
	}
}
impl Display for WeekNum {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

// ----

pub type AbsencesByWeek = HashMap<WeekNum, Vec<AbsenceRaw>>;
pub type AbsencesByWeekAndExcuse = HashMap<WeekNum, AbsencesByExcuse>;

pub fn split_by_week(iter: impl IntoIterator<Item = AbsenceRaw>) -> anyhow::Result<AbsencesByWeek> {
	let mut buf = HashMap::new();

	for absence in iter {
		let weeknum = WeekNum::from_date(absence.lesson.start_time);

		let mut existing: Vec<AbsenceRaw> = buf.remove(&weeknum).unwrap_or_default();
		existing.push(absence);
		buf.insert(weeknum, existing);
	}

	Ok(buf)
}

pub fn split_by_week_and_excuse(
	iter: impl IntoIterator<Item = AbsenceRaw>,
) -> anyhow::Result<AbsencesByWeekAndExcuse> {
	let by_week = split_by_week(iter).with_context(|| "failed to split by week")?;
	let by_excuse = by_week
		.into_iter()
		.map(|(week, data)| (week, crate::absences_by_excuse_type(data.iter())));

	Ok(by_excuse.collect())
}
