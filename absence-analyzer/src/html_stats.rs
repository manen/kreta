use std::{
	fmt::Display,
	hash::{DefaultHasher, Hash, Hasher},
};

use crate::{AbsenceDetails, AbsencesByExcuse, ExcuseType};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Color {
	r: u8,
	g: u8,
	b: u8,
}
impl Display for Color {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
	}
}

pub fn hash_to_color<T: Hash>(value: &T) -> Color {
	let mut hasher = DefaultHasher::new();
	value.hash(&mut hasher);
	let hash = hasher.finish(); // u64

	Color {
		r: (hash >> 24) as u8,
		g: (hash >> 16) as u8,
		b: (hash >> 8) as u8,
	}
}

pub fn by_excuse_type(data: &AbsencesByExcuse) -> String {
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
		let col_width = 5.0;
		let col_color = hash_to_color(ty);
		let col_text = std::format_args!("{:.1}", details.hours);

		let col = format!(
			"
			<div style=\"display: flex; align-items: center; flex-direction: column;\">
				<div style=\"display: flex; justify-content: center; color: black; margin: 0.5rem; margin-bottom: 0.3rem;\">
					{ty}
				</div>
				<div style=\"display: flex; justify-content: center; color: black; margin: 0.2rem; font-weight: bold;\">
					{col_text} óra
				</div>
				<div style=\"height: {col_height}em; width: {col_width}em; background-color: {col_color};\">
				</div>
			</div>"
		);
		col
	});
	let cols = cols.collect::<String>();

	let container = format!(
		"<div style=\"display: flex; flex-direction: row; justify-content: center; align-items: flex-start;\">{cols}</div>"
	);

	container
}
