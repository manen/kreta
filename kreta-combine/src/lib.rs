use std::borrow::Cow;

use anyhow::Context;
use kreta_rs::client::Client;

pub async fn get_combined(
	client: &Client,
	from: &str,
	to: &str,
) -> anyhow::Result<CombinedTimetable> {
	let timetable = client.timetable(from, to);
	let homework = client.homework(from, to);
	let exams = client.exams(from, to);

	let (timetable, homework, exams) = tokio::join!(timetable, homework, exams);
	let timetable = timetable
		.with_context(|| format!("while querying the timetable between {from} and {to}"))?;
	let homework =
		homework.with_context(|| format!("while querying homework between {from} and {to}"))?;
	let exams = exams.with_context(|| format!("while querying exams between {from} and {to}"))?;

	// timetable comes with everything about classes
	// homework contains deadline date & deadline lesson number
	// exam contains some extra info but the exam uid can be found in the timetable

	// the easiest and stupidest way to go about this is just loop over the lessons in the timetable
	// as usual and:
	// - check if an announced_exam_uid is present and find it in the exams if it is
	// - loop over all the homework to check if there's one for that class and that day // ! <- this would be slow as fuck probably

	// soooo maybe split up the homework into a hashmap of deadline dates beforehand so reads are fast

	// * ----

	// so to conclude

	// timetable stays as is
	// Vec<HomeworkRaw> -> HashMap<String, HomeworkRaw> (where key is deadline date & the subject name hashed or sum)
	// Vec<ExamRaw> -> HashMap<String, ExamRaw> (where key is exam uid)

	todo!()
}

type CombinedTimetable = Vec<CombinedLesson>;

#[derive(Clone, Debug)]
pub struct CombinedLesson {}

// actually i don't think i want to rewrite the whole lesson -> calendar pipeline
// so probably just extend timetable-to-ical::lesson_to_event with another arg that contains extra data
// like whether there's deadline homework present and the details of the announced exam (if any)

// then this crate is kinda just for shits and giggles but whatever kreta-combine sounds cool
