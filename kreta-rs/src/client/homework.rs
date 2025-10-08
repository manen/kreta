use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};

#[cfg(feature = "client")]
use super::Client;

#[cfg(feature = "client")]
impl Client {
	/// https://nzx.hu/kreta-api/mobileapi/gethomeworks \
	/// from & to are both yyyy-mm-dd
	/// maximum distance between from & to is 3 weeks
	pub async fn homework(&self, from: &str, to: &str) -> anyhow::Result<Vec<HomeworkRaw>> {
		let url = format!(
			"https://{}.e-kreta.hu/ellenorzo/v3/Sajat/HaziFeladatok?datumTol={from}&datumIg={to}",
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
		let resp: Vec<HomeworkRaw> = serde_json::from_str(&resp)
			.with_context(|| format!("failed to deserialize response from {url}"))?;

		Ok(resp)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// as returned by https://[instituteCode].e-kreta.hu/ellenorzo/v3/Sajat/HaziFeladatok
pub struct HomeworkRaw {
	#[serde(rename = "Uid")]
	pub uid: String,

	#[serde(rename = "Tantargy")]
	pub class: ClassRaw,
	#[serde(rename = "TantargyNeve")]
	pub class_name: String,

	#[serde(rename = "RogzitoTanarNeve")]
	pub teachers_name: String,
	#[serde(rename = "Szoveg")]
	pub text: String,

	#[serde(rename = "FeladasDatuma")]
	pub date_assigned: String,
	#[serde(rename = "HataridoDatuma")]
	pub date_deadline: String,
	#[serde(rename = "RogzitesIdopontja")]
	pub date_registered: String,

	#[serde(rename = "IsTanarRogzitette")]
	pub is_registered_by_teacher: bool,
	#[serde(rename = "IsTanuloHaziFeladatEnabled")]
	pub is_student_homework_enabled: bool,
	#[serde(rename = "IsMegoldva")]
	pub is_solved: bool,
	#[serde(rename = "IsBeadhato")]
	pub is_submittable: bool,
	#[serde(rename = "IsCsatolasEngedelyezes")]
	pub is_attachment_enabled: bool,

	#[serde(rename = "OsztalyCsoport")]
	pub class_group: ClassGroupRaw,
}
pub use super::timetable::ClassRaw;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// a different data type than the one used for timetables for some fucking reason
pub struct ClassGroupRaw {
	#[serde(rename = "Uid")]
	uid: String,
}
