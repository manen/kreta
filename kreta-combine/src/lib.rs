use std::collections::HashMap;

use anyhow::Context;
use kreta_rs::client::{
	Client, absences::AbsenceRaw, exam::ExamRaw, homework::HomeworkRaw, timetable::LessonRaw,
};

use crate::hash::{LessonHashByAbsence, LessonHashByHomework};

pub mod hash;
pub mod matching;
pub mod process;
pub mod receive;

pub type Preprocessed = (
	Vec<LessonRaw>,
	HashMap<LessonHashByHomework, HomeworkRaw>,
	HashMap<String, ExamRaw>,
	HashMap<LessonHashByAbsence, AbsenceRaw>,
);

#[derive(Clone, Debug)]
pub struct CombinedLesson {
	pub lesson_raw: LessonRaw,
	pub exam: Option<ExamRaw>,
	pub homework: Option<HomeworkRaw>,
	pub absence: Option<AbsenceRaw>,
}
type CombinedTimetable = Vec<CombinedLesson>;

// actually i don't think i want to rewrite the whole lesson -> calendar pipeline
// so probably just extend timetable-to-ical::lesson_to_event with another arg that contains extra data
// like whether there's deadline homework present and the details of the announced exam (if any)

// then this crate is kinda just for shits and giggles but whatever kreta-combine sounds cool

pub async fn fetch_combined_with_remainder(
	client: &Client,
	from: chrono::DateTime<chrono::Utc>,
	to: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<(
	Vec<CombinedLesson>,
	HashMap<LessonHashByHomework, HomeworkRaw>,
	HashMap<String, ExamRaw>,
)> {
	let preprocessed = receive::get_preprocessed_range(client, from, to)
		.await
		.with_context(|| "while calling receive::get_preprocessed_range")?;

	let result = matching::match_preprocessed_with_remainder(preprocessed)
		.with_context(|| "while calling matching::match_preprocessed_with_remainder")?;
	Ok(result)
}

/// if there's any homework or absences that don't get matched with their lesson of origin
/// they get thrown away by this function.
pub async fn fetch_combined(
	client: &Client,
	from: chrono::DateTime<chrono::Utc>,
	to: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<Vec<CombinedLesson>> {
	let preprocessed = receive::get_preprocessed_range(client, from, to)
		.await
		.with_context(|| "while calling receive::get_preprocessed_range")?;

	let result = matching::match_preprocessed(preprocessed)
		.with_context(|| "while calling matching::match_preprocessed")?;
	Ok(result)
}
