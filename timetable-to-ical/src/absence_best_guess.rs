#[derive(Copy, Clone, Debug)]
pub enum Absence {
	Present,
	Late,
	Absent,
}

pub fn absence_guess(name: &str) -> Absence {
	// a helyzet hogy nincs egy darab rendes hianyzasom eddig az evben szoval nemtom hogy a rendes hianyzast
	// minek hivja az api, ha ezt olvasod es tudod akk megkoszonom ha nyitsz egy pull requestet es ide beirod :)

	match name {
		"Jelenlet" => Absence::Present,
		// Na: nem definialt, the lesson hasn't happened yet
		"Na" => Absence::Present,
		"Keses" => Absence::Late,
		_ => Absence::Absent,
	}
}
