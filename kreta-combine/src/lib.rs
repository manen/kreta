use std::{
	collections::HashMap,
	hash::{DefaultHasher, Hash, Hasher},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Budapest;
use kreta_rs::client::{
	Client, absences::AbsenceRaw, exam::ExamRaw, homework::HomeworkRaw, timetable::LessonRaw,
};

fn get_homework_hash(date: &DateTime<Utc>, subject_uid: &str) -> anyhow::Result<u64> {
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
	HashMap<u64, AbsenceRaw>,
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
	let absences = async {
		let absences = client.absences(from, to).await?;
		let mut map = HashMap::new();
		process_absences(&mut map, absences)?;
		anyhow::Ok(map)
	};

	let (timetable, homework, exams, absences) = tokio::join!(timetable, homework, exams, absences);
	let timetable = timetable
		.with_context(|| format!("while querying the timetable between {from} and {to}"))?;
	let homework =
		homework.with_context(|| format!("while querying homework between {from} and {to}"))?;
	let exams = exams.with_context(|| format!("while querying exams between {from} and {to}"))?;
	let absences =
		absences.with_context(|| format!("while querying absences from {from} to {to}"))?;

	Ok((timetable, homework, exams, absences))
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
	let absences = async {
		let mut buf = HashMap::new();
		let mut stream = client.absences_range_stream(from.clone(), to.clone());
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading chunk from absences_range_stream")?;
			process_absences(&mut buf, next)
				.with_context(|| "while processing chunk from absences_range_stream")?;
		}
		anyhow::Ok(buf)
	};

	let (timetable, homework, exams, absences) = tokio::join!(timetable, homework, exams, absences);
	let (timetable, homework, exams, absences) = (
		timetable.with_context(|| format!("while querying lessons between {from:?} and {to:?}"))?,
		homework.with_context(|| format!("while querying homework between {from:?} and {to:?}"))?,
		exams.with_context(|| format!("while querying exams between {from:?} and {to:?}"))?,
		absences.with_context(|| format!("while querying absences between {from:?} and {to:?}"))?,
	);

	anyhow::Ok((timetable, homework, exams, absences))
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
				"failed to parse homework deadline date {} as DateTime<Utc>",
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
/// same logic as homework since no persistent id
fn process_absences(
	buf: &mut HashMap<u64, AbsenceRaw>,
	incoming: impl IntoIterator<Item = AbsenceRaw>,
) -> anyhow::Result<()> {
	let iter = incoming.into_iter().map(|absence| {
		let date: chrono::DateTime<Utc> = absence.lesson.start_time.parse().with_context(|| {
			format!(
				"failed to parse absence lesson start time {} as DateTime<Utc>",
				absence.lesson.start_time
			)
		})?;
		let hash = get_homework_hash(&date, &absence.subject.uid).with_context(|| {
			format!(
				"while calculating hash for {} absence from {}",
				absence.subject.name, absence.lesson.start_time
			)
		})?;
		Ok((hash, absence))
	});
	let iter = iter.collect::<anyhow::Result<Vec<_>>>()?;
	buf.extend(iter);
	Ok(())
}

type CombinedTimetable = Vec<CombinedLesson>;

#[derive(Clone, Debug)]
pub struct CombinedLesson {
	pub lesson_raw: LessonRaw,
	pub exam: Option<ExamRaw>,
	pub homework: Option<HomeworkRaw>,
	pub absence: Option<AbsenceRaw>,
}

fn match_preprocessed_internal(
	preprocessed: Preprocessed,
) -> anyhow::Result<(
	CombinedTimetable,
	HashMap<u64, HomeworkRaw>,
	HashMap<String, ExamRaw>,
	HashMap<u64, AbsenceRaw>,
)> {
	let (timetable, mut homework, mut exams, mut absences) = preprocessed;
	let mut combined = Vec::with_capacity(timetable.len());

	for lesson in timetable {
		let (homework, absence) = match &lesson.subject {
			Some(subject) => {
				let date: DateTime<Utc> = lesson.start_time.parse().with_context(|| {
					format!(
						"while parsing lesson start time {} as a DateTime",
						lesson.start_time
					)
				})?;

				let homework_hash = get_homework_hash(&date, &subject.uid)?;

				let homework = homework.remove(&homework_hash);
				let absence = absences.remove(&homework_hash);

				(homework, absence)
			}
			None => (None, None),
		};
		let exam = match &lesson.announced_exam_uid {
			Some(exam_uid) => exams.remove(exam_uid),
			None => None,
		};

		combined.push(CombinedLesson {
			lesson_raw: lesson,
			exam,
			homework,
			absence,
		});
	}

	Ok((combined, homework, exams, absences))
}

pub fn match_preprocessed(preprocessed: Preprocessed) -> anyhow::Result<CombinedTimetable> {
	let (timetable, _, _, _) = match_preprocessed_internal(preprocessed)?;
	Ok(timetable)
}
pub fn match_preprocessed_with_remainder(
	preprocessed: Preprocessed,
) -> anyhow::Result<(
	CombinedTimetable,
	HashMap<u64, HomeworkRaw>,
	HashMap<String, ExamRaw>,
)> {
	let (timetable, mut homework, mut exams, mut absences) =
		match_preprocessed_internal(preprocessed)?;
	homework.shrink_to_fit();
	exams.shrink_to_fit();
	absences.shrink_to_fit();

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
