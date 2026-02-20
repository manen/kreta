#[cfg(feature = "timerange")]
#[cfg(feature = "client")]
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[cfg(feature = "client")]
use crate::client::Client;
use crate::client::exam::{ClassGroupRaw, SubjectRaw, UidNameAndDescRaw};

#[cfg(feature = "client")]
impl Client {
	/// https://nzx.hu/kreta-api/mobileapi/getomissions \
	/// from & to are both yyyy-mm-dd
	/// maximum distance is 3 weeks
	pub async fn absences(&self, from: &str, to: &str) -> anyhow::Result<Vec<AbsenceRaw>> {
		// println!("fetching absences between {from} and {to}");

		use anyhow::Context;

		let url = format!(
			"https://{}.e-kreta.hu/ellenorzo/v3/sajat/Mulasztasok?datumTol={from}&datumIg={to}",
			self.inst_id
		);
		let req = self
			.client
			.get(&url)
			.bearer_auth(self.access_token())
			.build()?;

		let resp = self.client.execute(req).await?;
		let status_code = resp.status();
		if !status_code.is_success() {
			use anyhow::anyhow;

			let body = resp.text().await?;
			let err = anyhow!("{url} returned {status_code}\n{body}");
			return Err(err);
		}

		let resp = resp.text().await?;
		let resp: Vec<AbsenceRaw> = serde_json::from_str(&resp)
			.with_context(|| format!("failed to deserialize response from {url}"))?;

		Ok(resp)
	}
}

#[cfg(feature = "client")]
#[cfg(feature = "timerange")]
impl Client {
	/// creates a pollable stream of the absences in chunks
	/// needs to be polled to start doing anything
	pub fn absences_range_stream(
		&self,
		from: chrono::DateTime<Utc>,
		to: chrono::DateTime<Utc>,
	) -> impl futures::Stream<Item = anyhow::Result<Vec<AbsenceRaw>>> {
		use futures::stream::FuturesUnordered;

		let timesplit = timerange::range(
			from,
			to,
			chrono::Duration::weeks(3),
			chrono::Duration::days(1),
		);

		let mut stream = FuturesUnordered::new();
		stream.extend(timesplit.map(|(from, to)| async move {
			let from = from.format("%Y-%m-%d").to_string();
			let to = to.format("%Y-%m-%d").to_string();

			let absences = self.absences(&from, &to).await?;
			anyhow::Ok(absences)
		}));

		stream
	}

	/// absence query with no maximum distance between from & to
	pub async fn absences_range(
		&self,
		from: chrono::DateTime<Utc>,
		to: chrono::DateTime<Utc>,
	) -> anyhow::Result<Vec<AbsenceRaw>> {
		use futures::StreamExt as _;

		let mut buf = Vec::new();

		let mut stream = self.absences_range_stream(from, to);
		while let Some(next) = stream.next().await {
			use anyhow::Context;

			let next =
				next.with_context(|| format!("while reading absences from timerange stream"))?;
			buf.extend(next);
		}
		Ok(buf)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbsenceRaw {
	#[serde(rename = "KeszitesDatuma")]
	pub date_of_creation: String,
	#[serde(rename = "Datum")]
	pub date: String,
	#[serde(rename = "KesesPercben")]
	pub late_by_minutes: Option<i32>,
	#[serde(rename = "OsztalyCsoport")]
	pub class_group: ClassGroupRaw,

	#[serde(rename = "IgazolasAllapota")]
	pub excuse_status: String,
	#[serde(rename = "IgazolasTipusa")]
	pub excuse_type: Option<UidNameAndDescRaw>,

	#[serde(rename = "Ora")]
	pub lesson: LessonRaw,

	#[serde(rename = "Mod")]
	pub mode: UidNameAndDescRaw,
	#[serde(rename = "Tantargy")]
	pub subject: SubjectRaw,

	#[serde(rename = "RogzitoTanarNeve")]
	pub teachers_name: String,
	#[serde(rename = "Tipus")]
	pub typ: UidNameAndDescRaw,

	#[serde(rename = "Uid")]
	pub uid: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LessonRaw {
	#[serde(rename = "KezdoDatum")]
	pub start_time: String,
	#[serde(rename = "VegDatum")]
	pub end_time: String,
	#[serde(rename = "Oraszam")]
	pub oraszam: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeRaw {
	#[serde(rename = "Nev")]
	pub name: String,
	#[serde(rename = "Kategoria")]
	pub category: UidNameAndDescRaw,
	#[serde(rename = "Uid")]
	pub uid: String,
}
