use std::collections::HashMap;

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
		jó ezzel most nincs időm igazán foglalkozni de itt a dátum a hiányzás előtti nap nem aznap
		szóval ha pl október harmadikán van igazolatlanod itt október 2-t fog írni
		<p>{days}</p>
	</div>"
	);
	p
}

pub fn unexcused_amount_by_date(
	iter: impl Iterator<Item = AbsenceRaw>,
) -> HashMap<String, AbsenceDetails> {
	let mut days = HashMap::new();

	for absence in iter {
		let excuse_type = ExcuseType::derive_from(&absence);
		match excuse_type {
			Ok(ExcuseType::Unexcused | ExcuseType::ToBeExcused) => {
				// igazolatlan vagy igazolando szoval szamoljuk
			}
			_ => continue,
		}

		let day = absence
			.date
			.split('T')
			.next()
			.unwrap_or("absence.date is probably incorrect");

		crate::by_excuse::push_to_absence_buf(&mut days, day.to_owned(), absence.late_by_minutes);
	}

	days
}
