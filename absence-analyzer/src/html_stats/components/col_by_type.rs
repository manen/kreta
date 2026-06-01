use crate::{AbsenceDetails, AbsencesByExcuse, ExcuseType, html_stats::hash_to_color};

pub fn by_excuse_type(data: &AbsencesByExcuse) -> String {
	// sort types by absence hours
	let mut sorted = data.absences.iter().collect::<Vec<_>>();
	sorted.sort_by(|(_, a), (_, b)| {
		a.hours
			.partial_cmp(&b.hours)
			.unwrap_or(std::cmp::Ordering::Equal)
	});

	by_excuse_type_unsorted(sorted.iter().copied())
}

/// creates a component to be placed within an html frame
pub fn by_excuse_type_unsorted<'a>(
	data: impl Iterator<Item = (&'a ExcuseType, &'a AbsenceDetails)>,
) -> String {
	let cols = data.map(|(ty, details)| {
		let col_height = details.hours * 0.6;
		let col_color = hash_to_color(ty);
		let col_text = std::format_args!("{:.1}", details.hours);

		let col = format!(
			"
			     	<div style=\"display: flex; justify-content: center; color: var(--text-color); margin: 0.5rem; margin-bottom: 0.3rem;\"> {ty} </div>
			     	<div style=\"display: flex; justify-content: center; color: var(--text-color); margin: 0.2rem; font-weight: bold;\"> {col_text} óra </div>
			     	<div style=\"height: {col_height}em; width: max; background-color: {col_color};\"></div>
			     "
			// "
   //  <div> {ty} </div>
   //  <div> {col_text} óra </div>
   //  <div> {col_height} </div>
   //  "
		);
		col
	});
	let cols = cols.collect::<String>();

	let container = format!(
		"
			<div style=\"overflow: auto\">
				<div style=\"display: grid; grid-template-rows: repeat(3, auto); grid-auto-flow: column; gap: 0.5em;\">
					{cols}
				</div>
			</div>"
	);

	container
}

// vmi mukodik vmi nem
