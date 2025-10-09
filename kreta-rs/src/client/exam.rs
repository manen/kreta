use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};

#[cfg(feature = "client")]
use super::Client;

#[cfg(feature = "client")]
impl Client {
	/// https://nzx.hu/kreta-api/mobileapi/getannouncedtests \
	/// from & to are yyyy-mm-dd
	/// max distance is 1 month
	pub async fn exams(&self, from: &str, to: &str) -> anyhow::Result<Vec<ExamRaw>> {
		let url = format!(
			"https://{}.e-kreta.hu/ellenorzo/v3/sajat/BejelentettSzamonkeresek?datumTol={from}&datumIg={to}",
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
			let body = resp.text().await?;
			let err = anyhow!("{url} returned {status_code}\n{body}");
			return Err(err);
		}

		let resp = resp.text().await?;
		println!("{resp}");
		let resp: Vec<ExamRaw> = serde_json::from_str(&resp)
			.with_context(|| format!("failed to deserialize response from {url}"))?;

		Ok(resp)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExamRaw {
	#[serde(rename = "Uid")]
	pub uid: String,
	#[serde(rename = "Datum")]
	pub date: String,

	#[serde(rename = "BejelentesDatuma")]
	pub date_announced: String,
	#[serde(rename = "RogzitoTanarNeve")]
	pub teachers_name: String,

	#[serde(rename = "OrarendiOraOraszama")]
	/// for the given day (hanyadik ora aznap)
	pub lesson_index_in_timetable: i32,
	#[serde(rename = "Tantargy")]
	pub subject: SubjectRaw,
	#[serde(rename = "TantargyNeve")]
	pub subject_name: String,

	#[serde(rename = "Temaja")]
	pub topic: String,
	#[serde(rename = "Modja")]
	pub method: UidNameAndDescRaw,

	#[serde(rename = "OsztalyCsoport")]
	pub class_group: ClassGroupRaw,
}

pub use crate::client::homework::ClassGroupRaw;
pub use crate::client::timetable::{SubjectRaw, UidNameAndDescRaw};
