use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};

#[cfg(feature = "client")]
use super::Client;

#[cfg(feature = "client")]
impl Client {
	/// https://nzx.hu/kreta-api/mobileapi/getlessons \
	/// from & to are both expected to be in the format of yyyy-mm-dd \
	/// maximum distance between from & to is one month
	pub async fn timetable(&self, from: &str, to: &str) -> anyhow::Result<Vec<LessonRaw>> {
		let url = format!(
			"https://{}.e-kreta.hu/ellenorzo/v3/sajat/OrarendElemek?datumTol={from}&datumIg={to}",
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
		// println!("{resp}");
		let resp: Vec<LessonRaw> = serde_json::from_str(&resp)
			.with_context(|| format!("while deserializing response from {url}"))?;

		Ok(resp)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// represents a lesson returned by https://[instituteCode].e-kreta.hu/ellenorzo/v3/sajat/OrarendElem
pub struct LessonRaw {
	#[serde(rename = "Uid")]
	pub uid: String,
	#[serde(rename = "Datum")]
	pub date: String,
	#[serde(rename = "KezdetIdopont")]
	pub start_time: String,
	#[serde(rename = "VegIdopont")]
	pub end_time: String,
	#[serde(rename = "Nev")]
	pub name: String,

	#[serde(rename = "Oraszam")]
	pub oraszam: i32,
	#[serde(rename = "OraEvesSorszama")]
	pub ora_eves_sorszama: Option<i32>,

	#[serde(rename = "OsztalyCsoport")]
	pub class_group: ClassGroupRaw,
	#[serde(rename = "TanarNeve")]
	pub teachers_name: String,
	#[serde(rename = "Tantargy")]
	pub class: ClassRaw,
	#[serde(rename = "Tema")]
	pub topic: Option<String>,
	#[serde(rename = "TeremNeve")]
	pub room_name: String,

	#[serde(rename = "Tipus")]
	pub lesson_type: UidNameAndDescRaw,
	#[serde(rename = "TanuloJelenlet")]
	pub student_presence: UidNameAndDescRaw,
	#[serde(rename = "Allapot")]
	pub status: UidNameAndDescRaw,

	#[serde(rename = "HelyettesTanarNeve")]
	pub substitute_teacher_name: Option<String>,
	#[serde(rename = "HaziFeladatUid")]
	pub homework_uid: Option<String>,
	#[serde(rename = "BejelentettSzamonkeresUid")]
	pub announced_exam_uid: Option<String>,

	#[serde(rename = "Letrehozas")]
	pub created_at: String,
	#[serde(rename = "UtolsoModositas")]
	pub last_modified: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassGroupRaw {
	#[serde(rename = "Uid")]
	pub uid: String,
	#[serde(rename = "Nev")]
	pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassRaw {
	#[serde(rename = "Uid")]
	pub uid: String,
	#[serde(rename = "Nev")]
	pub name: String,
	#[serde(rename = "Kategoria")]
	pub category: UidNameAndDescRaw,
	#[serde(rename = "SortIndex")]
	pub sort_index: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// seems to be a reused data structure in the timetable responses
pub struct UidNameAndDescRaw {
	#[serde(rename = "Uid")]
	pub uid: String,
	#[serde(rename = "Nev")]
	pub name: String,
	#[serde(rename = "Leiras")]
	pub desc: String,
}
