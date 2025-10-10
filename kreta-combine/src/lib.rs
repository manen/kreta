use std::{
	collections::HashMap,
	hash::{DefaultHasher, Hash, Hasher},
};

use anyhow::{Context, anyhow};
use kreta_rs::client::{Client, exam::ExamRaw, homework::HomeworkRaw, timetable::LessonRaw};

fn get_homework_hash(full_date: &str, subject_name: &str) -> anyhow::Result<u64> {
	let deadline_date = full_date
		.split('T')
		.next()
		.ok_or_else(|| anyhow!("homework.deadline_date is not a valid date (can't split by T)"))?;

	let mut hasher = DefaultHasher::new();
	deadline_date.hash(&mut hasher);
	subject_name.hash(&mut hasher);
	let hash = hasher.finish();

	Ok(hash)
}

pub type Preprocessed = (
	Vec<LessonRaw>,
	HashMap<u64, HomeworkRaw>,
	HashMap<String, ExamRaw>,
);
pub async fn get_preprocessed(
	client: &Client,
	from: &str,
	to: &str,
) -> anyhow::Result<Preprocessed> {
	let timetable = async {
		let timetable = client.timetable(from, to).await;
		timetable
	};
	let homework = async {
		let homework_raw = client.homework(from, to).await?;

		let mut homework_map = HashMap::with_capacity(homework_raw.len());
		// order all the homework we got into a hashmap where:
		// - key is deadline date (without time) & subject name hasher
		// - value is the HomeworkRaw
		for homework in homework_raw {
			let hash = get_homework_hash(&homework.date_deadline, &homework.subject_name)
				.with_context(|| {
					format!(
						"while calculating hash for {} homework due {}",
						homework.subject_name, homework.date_deadline
					)
				})?;
			homework_map.insert(hash, homework);
		}

		anyhow::Ok(homework_map)
	};
	let exams = async {
		let exams_raw = client.exams(from, to).await?;

		let mut exams_map = HashMap::with_capacity(exams_raw.len());
		for exam in exams_raw {
			exams_map.insert(exam.uid.clone(), exam);
		}

		anyhow::Ok(exams_map)
	};

	let (timetable, homework, exams) = tokio::join!(timetable, homework, exams);
	let timetable = timetable
		.with_context(|| format!("while querying the timetable between {from} and {to}"))?;
	let homework =
		homework.with_context(|| format!("while querying homework between {from} and {to}"))?;
	let exams = exams.with_context(|| format!("while querying exams between {from} and {to}"))?;

	Ok((timetable, homework, exams))
}

type CombinedTimetable = Vec<CombinedLesson>;

#[derive(Clone, Debug)]
pub struct CombinedLesson {
	pub lesson_raw: LessonRaw,
	pub exam: Option<ExamRaw>,
	pub homework: Option<HomeworkRaw>,
}

pub fn match_preprocessed(preprocessed: Preprocessed) -> anyhow::Result<CombinedTimetable> {
	let (timetable, mut homework, mut exams) = preprocessed;
	let mut combined = Vec::with_capacity(timetable.len());

	for lesson in timetable {
		let homework = match &lesson.subject {
			Some(subject) => {
				let homework_hash = get_homework_hash(&lesson.start_time, &subject.name)?;
				let homework = homework.remove(&homework_hash);
				homework
			}
			None => None,
		};
		let exam = match &lesson.announced_exam_uid {
			Some(exam_uid) => exams.remove(exam_uid),
			None => None,
		};

		combined.push(CombinedLesson {
			lesson_raw: lesson,
			exam,
			homework,
		});
	}

	Ok(combined)
}

pub async fn get_combined(
	client: &Client,
	from: &str,
	to: &str,
) -> anyhow::Result<CombinedTimetable> {
	let preprocessed = get_preprocessed(client, from, to).await?;
	match_preprocessed(preprocessed)
}

// actually i don't think i want to rewrite the whole lesson -> calendar pipeline
// so probably just extend timetable-to-ical::lesson_to_event with another arg that contains extra data
// like whether there's deadline homework present and the details of the announced exam (if any)

// then this crate is kinda just for shits and giggles but whatever kreta-combine sounds cool
