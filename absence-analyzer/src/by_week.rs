use std::{collections::HashMap, fmt::Display};

use anyhow::Context;
use chrono::{DateTime, Datelike, Duration, Utc};
use kreta_rs::client::absences::AbsenceRaw;

use crate::retreive::last_september_first_expl;

/// denotes which week a date is in from september 1st
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WeekNum(u32);
impl WeekNum {
	pub fn from_date(date: chrono::DateTime<Utc>) -> Self {
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
	pub fn into_range_expl(
		self,
		sept_1: DateTime<Utc>,
	) -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
		let sept_1_monday = sept_1 - Duration::days(sept_1.weekday().num_days_from_monday() as _);

		let date_monday = sept_1_monday + Duration::weeks(self.0 as _);
		let date_sunday = date_monday + Duration::days(6);

		(date_monday, date_sunday)
	}

	pub fn take(self) -> u32 {
		self.0
	}
}
impl Display for WeekNum {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_weeknum() {
		let base: DateTime<Utc> = "2025-10-03T12:00:00Z".parse().unwrap();
		let sept_1 = last_september_first_expl(base);

		let iter = 0..500;
		let iter = iter.map(|n| base + Duration::days(n));

		for date in iter {
			let weeknum = WeekNum::from_date(date);
			let (from, to) = weeknum.into_range_expl(sept_1);

			println!("{} < {} < {}    <- this should be true", from, date, to);
			assert!(!(date < from || date > to))
		}
	}
}

// ----

pub fn split_by_week(
	iter: impl IntoIterator<Item = AbsenceRaw>,
) -> anyhow::Result<HashMap<WeekNum, Vec<AbsenceRaw>>> {
	let mut buf = HashMap::new();

	for absence in iter {
		let date: chrono::DateTime<Utc> = absence
			.lesson
			.start_time
			.parse()
			.with_context(|| format!("failed to parse {} as DateTime<Utc>", absence.date))?;
		let weeknum = WeekNum::from_date(date);

		let mut existing: Vec<AbsenceRaw> = buf.remove(&weeknum).unwrap_or_default();
		existing.push(absence);
		buf.insert(weeknum, existing);
	}

	Ok(buf)
}
