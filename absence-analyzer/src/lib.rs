use std::{collections::HashMap, fmt::Display, hash::Hash};

use anyhow::{Context, anyhow};
use kreta_rs::client::{Client, absences::AbsenceRaw};

pub mod html_stats;
pub mod retreive;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Default)]
pub struct AbsenceDetails {
	occurrances: i32,
	hours: f32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum ExcuseType {
	#[default]
	ToBeExcused,
	Unexcused,
	Excused(String),
}
impl Display for ExcuseType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ToBeExcused => write!(f, "Igazolandó"),
			Self::Unexcused => write!(f, "Igazolatlan"),
			Self::Excused(excuse) => write!(f, "{excuse}"),
		}
	}
}
impl ExcuseType {
	pub fn derive_from(absence: &AbsenceRaw) -> anyhow::Result<Self> {
		match absence.excuse_status.as_ref() {
			"Igazolt" => match &absence.excuse_type {
				Some(typ) => Ok(Self::Excused(typ.desc.clone())),
				None => Err(anyhow!(
					"Excused absence doesn't have excuse_type\n{absence:#?}"
				)),
			},
			"Igazolando" => Ok(Self::ToBeExcused),
			"Igazolatlan" => Ok(Self::Unexcused),
			_ => Err(anyhow!(
				"failed to derive excuse type: {}\n{absence:#?}",
				absence.excuse_status
			)),
		}
	}
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct AbsencesByExcuse {
	absences: HashMap<ExcuseType, AbsenceDetails>,
}

fn push_to_absence_buf<K: Hash + Eq>(
	buf: &mut HashMap<K, AbsenceDetails>,
	typ: K,
	minutes: Option<i32>,
) {
	let existing = buf.remove(&typ);
	let mut existing = existing.unwrap_or_default();

	let hours = match minutes {
		Some(a) => a as f32 / 45.0,
		None => 1.0,
	};

	existing.occurrances += 1;
	existing.hours += hours;

	buf.insert(typ, existing);
}

pub fn absences_by_excuse_type_opt<'a>(
	absences: impl Iterator<Item = &'a AbsenceRaw>,
) -> HashMap<Option<ExcuseType>, AbsenceDetails> {
	let mut buf = HashMap::new();

	for absence in absences {
		let excuse_type = ExcuseType::derive_from(&absence);
		let excuse_type = match excuse_type {
			Ok(a) => Some(a),
			Err(err) => {
				eprintln!("{err}");
				None
			}
		};

		push_to_absence_buf(&mut buf, excuse_type, absence.late_by_minutes);
	}
	buf
}
pub fn absences_by_excuse_type<'a>(
	absences: impl Iterator<Item = &'a AbsenceRaw>,
) -> AbsencesByExcuse {
	let opt = absences_by_excuse_type_opt(absences);
	let absences = opt
		.into_iter()
		.filter_map(|(k, v)| match k {
			Some(a) => Some((a, v)),
			None => None,
		})
		.collect();
	AbsencesByExcuse { absences }
}
