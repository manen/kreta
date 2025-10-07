#[derive(Copy, Clone, Debug)]
pub enum Absence {
	Present,
	Late,
	Absent,
}

pub fn absence_guess(name: &str) -> Absence {
	// self-explanatory
	match name {
		"Jelenlet" => Absence::Present,
		// Na: nem definialt, the lesson hasn't happened yet
		"Na" => Absence::Present,
		"Keses" => Absence::Late,
		"Hianyzas" => Absence::Absent,
		_ => Absence::Absent,
	}
}
