use std::collections::HashSet;

use crate::{
	by_week::{AbsencesByWeekAndExcuse, WeekNum},
	html_stats::hash_to_color,
};

/// get from crate::by_week::split_by_week_and_excuse
pub fn line_by_week(data: &AbsencesByWeekAndExcuse) -> String {
	let all_excuse_types = data
		.iter()
		.flat_map(|(_, a)| a.absences.iter())
		.map(|(a, _)| a.clone())
		.collect::<HashSet<_>>();

	let highest_hours_in_a_week = data
		.iter()
		.flat_map(|(_, a)| a.absences.iter())
		.map(|(_, a)| a.hours)
		.reduce(|acc, a| if a > acc { a } else { acc })
		.unwrap_or_default()
		.ceil();

	let highest_weeknum = data
		.iter()
		.map(|(a, _)| *a)
		.max()
		.map(WeekNum::take)
		.unwrap_or_default();

	// sort
	let mut by_weeks_sorted = data.iter().collect::<Vec<_>>();
	by_weeks_sorted.sort_by(|(a, _), (b, _)| a.cmp(&b));

	// --

	// svg
	let view_box = "0 0 100 100";

	// for all excuse types, check every week for how many hours of that type happened
	// if none, put zero
	//
	// if done, turn the weeknum and hours into coordinates and create the polyline svg element
	let lines = all_excuse_types.iter().cloned().map(|excuse_type| {
		let points = by_weeks_sorted
			.iter()
			.map(|(weeknum, absences)| {
				let hours = absences
					.absences
					.get(&excuse_type)
					.map(|a| a.hours)
					.unwrap_or_default();
				(*weeknum, hours)
			})
			.map(|(weeknum, hours)| {
				format!(
					"{},{}",
					(weeknum.take() as f32 / highest_weeknum as f32) * 100.0,
					100.0 - ((hours as f32 / highest_hours_in_a_week as f32) * 100.0)
				)
			})
			.collect::<Vec<String>>()
			.join(" ");

		let fill = "none";
		let color = hash_to_color(&excuse_type);
		let width = 1;
		let points = points;

		let polyline = format!(
			"<polyline fill=\"{fill}\" stroke=\"{color}\" stroke-width=\"{width}\" points=\"{points}\" />"
		);
		polyline
	});
	let lines = lines.collect::<String>();

	let svg = format!(
		"
		<div style=\"display: flex; align-items: center; flex-direction: column; margin: 1rem;\">
			<svg viewBox=\"{view_box}\" style=\"width: 80rem; max-width: 100%; max-height: 100%;\">
				{lines}
			</svg>
		</div>
		"
	);

	svg
}
