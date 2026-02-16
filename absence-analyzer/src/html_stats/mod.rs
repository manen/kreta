use std::{
	fmt::Display,
	hash::{DefaultHasher, Hash, Hasher},
};

mod col_by_type;
pub use col_by_type::*;
mod total;
pub use total::*;

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
