use std::collections::HashMap;

use kreta_rs::client::{
	absences::AbsenceRaw, exam::ExamRaw, homework::HomeworkRaw, timetable::LessonRaw,
};

use crate::hash::{LessonHashByAbsence, LessonHashByHomework};

pub fn process_timetable(buf: &mut Vec<LessonRaw>, incoming: impl IntoIterator<Item = LessonRaw>) {
	buf.extend(incoming);
}
/// order all the homework we got into a hashmap where:
/// - key is deadline date (without time) & subject name hasher
/// - value is the HomeworkRaw
pub fn process_homework(
	buf: &mut HashMap<LessonHashByHomework, HomeworkRaw>,
	incoming: impl IntoIterator<Item = HomeworkRaw>,
) -> anyhow::Result<()> {
	let with_hash = incoming.into_iter().map(|hw| {
		let hash = LessonHashByHomework::new_from_homework(&hw);
		Ok((hash, hw))
	});
	let iter = with_hash.collect::<anyhow::Result<Vec<_>>>()?;
	buf.extend(iter);
	Ok(())
}
pub fn process_exams(
	buf: &mut HashMap<String, ExamRaw>,
	incoming: impl IntoIterator<Item = ExamRaw>,
) {
	let iter = incoming.into_iter().map(|exam| (exam.uid.clone(), exam));
	buf.extend(iter);
}
/// same logic as homework since no persistent id
pub fn process_absences(
	buf: &mut HashMap<LessonHashByAbsence, AbsenceRaw>,
	incoming: impl IntoIterator<Item = AbsenceRaw>,
) -> anyhow::Result<()> {
	let with_hash = incoming.into_iter().map(|absence| {
		let hash = LessonHashByAbsence::new_from_absence(&absence.lesson);
		Ok((hash, absence))
	});
	let iter = with_hash.collect::<anyhow::Result<Vec<_>>>()?;
	buf.extend(iter);
	Ok(())
}
