use std::{
	fmt::Display,
	hash::{DefaultHasher, Hash, Hasher},
};

use crate::AbsencesByExcuse;

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

/// creates a component to be placed within an html frame
pub fn by_excuse_type(data: &AbsencesByExcuse) -> String {
	let cols = data.absences.iter().map(|(ty, details)| {
		let col_height = (details.hours * 0.6) as i32;
		let col_width = 5.0;
		let col_color = hash_to_color(ty);
		let col_text = std::format_args!("{:.1}", details.hours);

		let col = format!(
			"
			<div style=\"display: flex; justify-content: center; flex-direction: column;\">
				<div style=\"display: flex; justify-content: center; color: black; padding: 0.5rem;\">
					{ty}
				</div>
				<div style=\"height: {col_height}em; width: {col_width}em; background-color: {col_color};\">
					<div style=\"display: flex; justify-content: center; color: white;\">
						{col_text}
						</div>
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
