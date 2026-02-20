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
	let view_width = 100.0;
	let view_height = 50.0;
	let view_box = "0 0 100 100";

	let weeknum_to_x = |weeknum| (weeknum as f32 / highest_weeknum as f32) * view_width;
	let hours_to_y =
		|hours| view_height - ((hours as f32 / highest_hours_in_a_week as f32) * view_height);

	// for all excuse types, check every week for how many hours of that type happened
	// if none, put zero
	//
	// if done, turn the weeknum and hours into coordinates and create the polyline svg element
	let data_points = all_excuse_types.iter().cloned().map(|excuse_type| {
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
				format!("{},{}", weeknum_to_x(weeknum.take()), hours_to_y(hours))
			})
			.collect::<Vec<String>>()
			.join(" ");

		let fill = "none";
		let color = hash_to_color(&excuse_type);
		let width = 0.7;
		let points = points;

		let polyline = format!(
			"<polyline fill=\"{fill}\" stroke=\"{color}\" stroke-width=\"{width}\" points=\"{points}\" />"
		);
		polyline
	});
	let data_points = data_points.collect::<String>();

	let undescriptive_gray = "rgb(107,107,107)";
	let undescriptive_gray_darker = "rgb(67,67,67)";
	let weeknum_guidelines = (0..highest_weeknum).map(|weeknum| {
		let x = weeknum_to_x(weeknum);

		let fill = "none";
		let color = undescriptive_gray_darker;
		// let width = if weeknum % 10 != 0 { 0.025 } else { 0.05 };
		let width = 0.025;
		let points = format!("{x},0 {x},{view_height}");

		format!(
			"<polyline fill=\"{fill}\" stroke=\"{color}\" stroke-width=\"{width}\" points=\"{points}\" />"
		)
	});
	let hour_guideline_frequency = 5;
	let hour_guidelines = (0..(highest_hours_in_a_week as i32 / hour_guideline_frequency)
		+ hour_guideline_frequency)
		.map(|hour| {
			let hour = hour * hour_guideline_frequency;
			let y = hours_to_y(hour as f32);

			let fill = "none";
			let color = undescriptive_gray;
			let width = if hour % 10 == 0 { 0.06 } else { 0.015 };
			let points = format!("0,{y} {view_width},{y}");

			format!(
				"<polyline fill=\"{fill}\" stroke=\"{color}\" stroke-width=\"{width}\" points=\"{points}\" />"
			)
		});
	let guidelines = weeknum_guidelines
		.chain(hour_guidelines)
		.collect::<String>();

	let svg = format!(
		"
		<div style=\"display: flex; align-items: center; flex-direction: column; margin: 1rem;\">
			<svg viewBox=\"{view_box}\" style=\"width: 80rem; max-width: 100%; max-height: 100%;\">
				{guidelines}
				{data_points}
			</svg>
		</div>
		"
	);

	svg
}
