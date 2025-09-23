use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};

use super::Client;

impl Client {
	/// https://nzx.hu/kreta-api/mobileapi/getlessons \
	/// from & to are both expected to be in the format of yyyy-mm-dd
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
		println!("{resp}");
		let resp: Vec<LessonRaw> = serde_json::from_str(&resp)
			.with_context(|| format!("while deserializing response from {url}"))?;

		Ok(resp)
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// represents a lesson returned by https://[instituteCode].e-kreta.hu/ellenorzo/v3/sajat/OrarendElem
pub struct LessonRaw {
	#[serde(rename = "Uid")]
	uid: String,
	#[serde(rename = "Datum")]
	date: String,
	#[serde(rename = "KezdetIdopont")]
	start_time: String,
	#[serde(rename = "VegIdopont")]
	end_time: String,
	#[serde(rename = "Nev")]
	name: String,

	#[serde(rename = "Oraszam")]
	oraszam: i32,
	#[serde(rename = "OraEvesSorszama")]
	ora_eves_sorszama: i32,

	#[serde(rename = "OsztalyCsoport")]
	class_group: ClassGroupRaw,
	#[serde(rename = "TanarNeve")]
	teachers_name: String,
	#[serde(rename = "Tantargy")]
	class: ClassRaw,
	#[serde(rename = "Tema")]
	topic: Option<String>,
	#[serde(rename = "TeremNeve")]
	room_name: String,

	#[serde(rename = "Tipus")]
	lesson_type: UidNameAndDescRaw,
	#[serde(rename = "TanuloJelenlet")]
	student_presence: UidNameAndDescRaw,
	#[serde(rename = "Allapot")]
	status: UidNameAndDescRaw,

	#[serde(rename = "HelyettesTanarNeve")]
	substitute_teacher_name: Option<String>,
	#[serde(rename = "HaziFeladatUid")]
	homework_uid: Option<String>,
	#[serde(rename = "BejelentettSzamonkeresUid")]
	announced_exam_uid: Option<String>,

	#[serde(rename = "Letrehozas")]
	created_at: String,
	#[serde(rename = "UtolsoModositas")]
	last_modified: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassGroupRaw {
	#[serde(rename = "Uid")]
	uid: String,
	#[serde(rename = "Nev")]
	name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassRaw {
	#[serde(rename = "Uid")]
	uid: String,
	#[serde(rename = "Nev")]
	name: String,
	#[serde(rename = "Kategoria")]
	category: UidNameAndDescRaw,
	#[serde(rename = "SortIndex")]
	sort_index: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// seems to be a reused data structure in the timetable responses
pub struct UidNameAndDescRaw {
	#[serde(rename = "Uid")]
	uid: String,
	#[serde(rename = "Nev")]
	name: String,
	#[serde(rename = "Leiras")]
	desc: String,
}
