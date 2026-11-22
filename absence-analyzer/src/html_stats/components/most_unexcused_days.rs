use std::collections::HashMap;

use chrono::NaiveDate;

use crate::{AbsenceDetails, AbsenceRaw, ExcuseType};

pub fn most_unexcused_days_html(iter: impl Iterator<Item = AbsenceRaw>) -> String {
	let days = unexcused_amount_by_date(iter);
	let mut days = days.into_iter().collect::<Vec<_>>();
	days.sort_by(|(_, a), (_, b)| b.hours.total_cmp(&a.hours));

	let days = days
		.iter()
		.map(|(date, details)| format!("<div>{date}: {:.2} óra</div>\n", details.hours))
		.collect::<String>();

	let p = format!(
		"
	<div>
		<p>{days}</p>
	</div>"
	);
	p
}

pub fn unexcused_amount_by_date(
	iter: impl Iterator<Item = AbsenceRaw>,
) -> HashMap<NaiveDate, AbsenceDetails> {
	let mut days = HashMap::new();

	for absence in iter {
		let excuse_type = ExcuseType::derive_from(&absence);
		match excuse_type {
			Ok(ExcuseType::Unexcused | ExcuseType::ToBeExcused) => {
				// igazolatlan vagy igazolando szoval szamoljuk
				let day = absence.lesson.start_time.date_naive();
				crate::by_excuse::push_to_absence_buf(
					&mut days,
					day.to_owned(),
					absence.late_by_minutes,
				);
			}
			_ => continue,
		}
	}

	days
}
