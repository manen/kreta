use serde::{Deserialize, Serialize};

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
	pub oraszam: Option<i32>,
	#[serde(rename = "OraEvesSorszama")]
	pub ora_eves_sorszama: Option<i32>,

	#[serde(rename = "OsztalyCsoport")]
	pub class_group: Option<ClassGroupRaw>,
	#[serde(rename = "TanarNeve")]
	pub teachers_name: Option<String>,
	#[serde(rename = "Tantargy")]
	pub subject: Option<SubjectRaw>,
	#[serde(rename = "Tema")]
	pub topic: Option<String>,
	#[serde(rename = "TeremNeve")]
	pub room_name: Option<String>,

	#[serde(rename = "Tipus")]
	pub lesson_type: UidNameAndDescRaw,
	#[serde(rename = "TanuloJelenlet")]
	pub student_presence: Option<UidNameAndDescRaw>,
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
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct SubjectRaw {
	#[serde(rename = "Uid")]
	pub uid: String,
	#[serde(rename = "Nev")]
	pub name: String,
	#[serde(rename = "Kategoria")]
	pub category: UidNameAndDescRaw,
	#[serde(rename = "SortIndex")]
	pub sort_index: i32,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
/// seems to be a reused data structure in the timetable responses
pub struct UidNameAndDescRaw {
	#[serde(rename = "Uid")]
	pub uid: String,
	#[serde(rename = "Nev")]
	pub name: String,
	#[serde(rename = "Leiras")]
	pub desc: String,
}

// ---

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct AbsenceRaw {
	#[serde(rename = "KeszitesDatuma")]
	pub date_of_creation: String,
	#[serde(rename = "Datum")]
	pub date: String,
	#[serde(rename = "KesesPercben")]
	pub late_by_minutes: Option<i32>,
	#[serde(rename = "OsztalyCsoport")]
	pub class_group: ClassGroupRawWithoutName,

	#[serde(rename = "IgazolasAllapota")]
	pub excuse_status: String,
	#[serde(rename = "IgazolasTipusa")]
	pub excuse_type: Option<UidNameAndDescRaw>,

	#[serde(rename = "Ora")]
	pub lesson: BriefLessonRaw,

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
/// A shorter version of a struct also called LessonRaw in crate::timetable
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct BriefLessonRaw {
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

// ---

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
	pub class_group: ClassGroupRawWithoutName,
}

// ---

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
/// as returned by https://[instituteCode].e-kreta.hu/ellenorzo/v3/Sajat/HaziFeladatok
pub struct HomeworkRaw {
	#[serde(rename = "Uid")]
	pub uid: String,

	#[serde(rename = "Tantargy")]
	pub subject: SubjectRaw,
	#[serde(rename = "TantargyNeve")]
	pub subject_name: String,

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
	pub class_group: ClassGroupRawWithoutName,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
/// a different data type than the one used for timetables for some fucking reason
pub struct ClassGroupRawWithoutName {
	#[serde(rename = "Uid")]
	uid: String,
}

impl HomeworkRaw {
	/// normally, homework text is an html fragment, usuall containing <p>, <a>, and <div> tags.
	/// this method scrapes out only the text, making it easier on the eyes and easier to read
	pub fn text_extract(&self) -> String {
		use scraper::Html;

		let fragment = Html::parse_fragment(&self.text);
		let text: String = fragment
			.root_element()
			.text()
			.collect::<Vec<_>>()
			.join("\n");

		text
	}
}
