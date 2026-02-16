use chrono::Utc;

use crate::{AbsencesByExcuse, ExcuseType};

const SCHOOL_YEAR_MONTHS: f32 = 9.5;

pub fn months_since_september_first() -> f32 {
	let september_first = crate::retreive::last_september_first();
	let now = Utc::now();

	let distance = now - september_first;
	let days = distance.num_days() as f32;
	let months = days / 30.436875;

	months
}

/// generic, input any hours of absences and it'll forecast how much of that absence
/// type you'll have by the end of the year
pub fn forecast_value(value: f32) -> f32 {
	let months = months_since_september_first();
	let per_month = value / months;
	let by_end_of_year = per_month * SCHOOL_YEAR_MONTHS;
	by_end_of_year
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UnexcusedForecast {
	pub only_unexcused: f32,
	pub with_to_be_excused: f32,
}

/// returns how many hours of unexcused absences you're likely to have by end of year
pub fn extract_unexcused_forecast(data: &AbsencesByExcuse) -> Option<UnexcusedForecast> {
	let to_be_excused = data.absences.get(&ExcuseType::ToBeExcused).map(|a| a.hours);
	let unexcused = data.absences.get(&ExcuseType::Unexcused).map(|a| a.hours);

	match unexcused {
		Some(unexcused) => {
			let forecast_unexcused_only = forecast_value(unexcused);
			let forecast_with_to_be_excused = match to_be_excused {
				None => forecast_unexcused_only,
				Some(to_be_excused) => forecast_value(unexcused + to_be_excused),
			};

			Some(UnexcusedForecast {
				only_unexcused: forecast_unexcused_only,
				with_to_be_excused: forecast_with_to_be_excused,
			})
		}
		None => None,
	}
}
