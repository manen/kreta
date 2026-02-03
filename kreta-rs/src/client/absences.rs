use serde::{Deserialize, Serialize};

#[cfg(feature = "client")]
use crate::client::Client;
use crate::client::exam::{ClassGroupRaw, SubjectRaw, UidNameAndDescRaw};

#[cfg(feature = "client")]
impl Client {
	pub async fn absences(&self, from: &str, to: &str) -> anyhow::Result<Vec<AbsenceRaw>> {
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
