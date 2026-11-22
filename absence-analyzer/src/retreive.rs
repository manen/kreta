use crate::AbsenceRaw;
use anyhow::Context;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use kreta_rs::client::Client;

pub fn last_september_first() -> NaiveDate {
	let now = Local::now();
	last_september_first_expl(now)
}
pub fn last_september_first_expl(now: DateTime<Local>) -> NaiveDate {
	let year = if now.month() > 9 || (now.month() == 9 && now.day() >= 1) {
		now.year()
	} else {
		now.year() - 1
	};

	NaiveDate::from_ymd_opt(year, 9, 1)
		.unwrap_or_else(|| panic!("last september 1st doesn't exist (ymd {}, 9, 1)", year))
}

pub async fn fetch_absences(client: &Client) -> anyhow::Result<Vec<AbsenceRaw>> {
	let from = last_september_first();

	let absences = client
		.absences_range(from, Local::now().date_naive())
		.await
		.with_context(|| format!("failed to query all absences since {}", from))?;

	Ok(absences)
}

#[cfg(feature = "save_load")]
mod save_load {
	use std::{io::ErrorKind, path::Path};

	use super::*;
	pub fn serialize_absences(absences: &[AbsenceRaw]) -> anyhow::Result<Vec<u8>> {
		let bin = rmp_serde::encode::to_vec(absences)?;
		Ok(bin)
	}
	pub fn deserialize_absences(bin: &[u8]) -> anyhow::Result<Vec<AbsenceRaw>> {
		let buf: Vec<AbsenceRaw> = rmp_serde::decode::from_slice(&bin)?;
		Ok(buf)
	}

	pub async fn save(absences: &[AbsenceRaw], path: impl AsRef<Path>) -> anyhow::Result<()> {
		let bin = serialize_absences(absences)?;
		tokio::fs::write(path, bin).await?;
		Ok(())
	}
	pub async fn load(path: impl AsRef<Path>) -> anyhow::Result<Option<Vec<AbsenceRaw>>> {
		let bin = tokio::fs::read(path).await;
		let bin = match bin {
			Ok(bin) => bin,
			Err(err) => match err.kind() {
				ErrorKind::NotFound => return Ok(None),
				_ => return Err(err.into()),
			},
		};

		let a = deserialize_absences(&bin)?;
		Ok(Some(a))
	}
}
#[cfg(feature = "save_load")]
pub use save_load::*;
