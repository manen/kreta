use std::{
	collections::HashMap,
	hash::{DefaultHasher, Hash, Hasher},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Budapest;
use kreta_rs::client::{Client, exam::ExamRaw, homework::HomeworkRaw, timetable::LessonRaw};

fn get_homework_hash<Tz: chrono::TimeZone>(
	date: &DateTime<Tz>,
	subject_uid: &str,
) -> anyhow::Result<u64> {
	let deadline_date = date.with_timezone(&Budapest).date_naive();

	let mut hasher = DefaultHasher::new();
	deadline_date.hash(&mut hasher);
	subject_uid.hash(&mut hasher);
	let hash = hasher.finish();

	Ok(hash)
}

pub type Preprocessed = (
	Vec<LessonRaw>,
	HashMap<u64, HomeworkRaw>,
	HashMap<String, ExamRaw>,
);

/// simple, 3 weeks at max
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

		let mut homework_map = HashMap::new();
		process_homework(&mut homework_map, homework_raw)?;
		anyhow::Ok(homework_map)
	};
	let exams = async {
		let exams_raw = client.exams(from, to).await?;

		let mut exams_map = HashMap::new();
		process_exams(&mut exams_map, exams_raw);
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

#[cfg(feature = "timerange")]
pub async fn get_preprocessed_range(
	client: &Client,
	from: chrono::DateTime<chrono::Utc>,
	to: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<Preprocessed> {
	use futures::StreamExt;
	let timetable = async {
		let mut buf = Vec::new();
		let mut stream = client.timetable_range_stream(from.clone(), to.clone());
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading chunk from timetable_range_stream")?;
			process_timetable(&mut buf, next);
		}

		anyhow::Ok(buf)
	};
	let homework = async {
		let mut buf = HashMap::new();
		let mut stream = client.homework_range_stream(from.clone(), to.clone());
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading chunk from homework_range_stream")?;
			process_homework(&mut buf, next)
				.with_context(|| "while processing chunk from homework_range_stream")?;
		}

		anyhow::Ok(buf)
	};
	let exams = async {
		let mut buf = HashMap::new();
		let mut stream = client.exams_range_stream(from.clone(), to.clone());
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading chunk from exams_range_stream")?;
			process_exams(&mut buf, next);
		}

		anyhow::Ok(buf)
	};

	let (timetable, homework, exams) = tokio::join!(timetable, homework, exams);
	let (timetable, homework, exams) = (
		timetable.with_context(|| format!("while querying lessons between {from:?} and {to:?}"))?,
		homework.with_context(|| format!("while querying homework between {from:?} and {to:?}"))?,
		exams.with_context(|| format!("while querying exams between {from:?} and {to:?}"))?,
	);

	anyhow::Ok((timetable, homework, exams))
}

fn process_timetable(buf: &mut Vec<LessonRaw>, incoming: impl IntoIterator<Item = LessonRaw>) {
	buf.extend(incoming);
}
/// order all the homework we got into a hashmap where:
/// - key is deadline date (without time) & subject name hasher
/// - value is the HomeworkRaw
fn process_homework(
	buf: &mut HashMap<u64, HomeworkRaw>,
	incoming: impl IntoIterator<Item = HomeworkRaw>,
) -> anyhow::Result<()> {
	let iter = incoming.into_iter().map(|hw| {
		let deadline_date: DateTime<Utc> = hw.date_deadline.parse().with_context(|| {
			format!(
				"failed to parse homework deadline date {} as DateTime",
				hw.date_deadline
			)
		})?;
		let hash = get_homework_hash(&deadline_date, &hw.subject.uid).with_context(|| {
			format!(
				"while calculating hash for {} homework due {}",
				hw.subject_name, hw.date_deadline
			)
		})?;
		Ok((hash, hw))
	});
	let iter = iter.collect::<anyhow::Result<Vec<_>>>()?;
	buf.extend(iter);
	Ok(())
}
fn process_exams(buf: &mut HashMap<String, ExamRaw>, incoming: impl IntoIterator<Item = ExamRaw>) {
	let iter = incoming.into_iter().map(|exam| (exam.uid.clone(), exam));
	buf.extend(iter);
}

type CombinedTimetable = Vec<CombinedLesson>;

#[derive(Clone, Debug)]
pub struct CombinedLesson {
	pub lesson_raw: LessonRaw,
	pub exam: Option<ExamRaw>,
	pub homework: Option<HomeworkRaw>,
}

fn match_preprocessed_internal(
	preprocessed: Preprocessed,
) -> anyhow::Result<(
	CombinedTimetable,
	HashMap<u64, HomeworkRaw>,
	HashMap<String, ExamRaw>,
)> {
	let (timetable, mut homework, mut exams) = preprocessed;
	let mut combined = Vec::with_capacity(timetable.len());

	for lesson in timetable {
		let homework = match &lesson.subject {
			Some(subject) => {
				let date: DateTime<Utc> = lesson.start_time.parse().with_context(|| {
					format!(
						"while parsing lesson start time {} as a DateTime",
						lesson.start_time
					)
				})?;

				let homework_hash = get_homework_hash(&date, &subject.uid)?;
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

	Ok((combined, homework, exams))
}

pub fn match_preprocessed(preprocessed: Preprocessed) -> anyhow::Result<CombinedTimetable> {
	let (timetable, _, _) = match_preprocessed_internal(preprocessed)?;
	Ok(timetable)
}
pub fn match_preprocessed_with_remainder(
	preprocessed: Preprocessed,
) -> anyhow::Result<(
	CombinedTimetable,
	HashMap<u64, HomeworkRaw>,
	HashMap<String, ExamRaw>,
)> {
	let (timetable, mut homework, mut exams) = match_preprocessed_internal(preprocessed)?;
	homework.shrink_to_fit();
	exams.shrink_to_fit();

	Ok((timetable, homework, exams))
}

/// 3 weeks max
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
