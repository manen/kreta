use crate::AbsencesByExcuse;

/// creates a component to be placed within an html frame
pub fn by_excuse_type(data: &AbsencesByExcuse) -> String {
	let iter = data.absences.iter().map(|(ty, details)| {
		let col_height = (details.hours * 10.0) as i32;
		let col_width = 5.0;
		let col_div = format!("<div style=\"height: {col_height}em; width: {col_width}em\"></div>");
	});
}
