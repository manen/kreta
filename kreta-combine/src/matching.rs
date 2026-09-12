use std::collections::HashMap;

use kreta_rs::client::{absences::AbsenceRaw, exam::ExamRaw, homework::HomeworkRaw};

use crate::{
	CombinedLesson, CombinedTimetable, Preprocessed,
	hash::{LessonHashByAbsence, LessonHashByHomework},
};

fn match_preprocessed_internal(
	preprocessed: Preprocessed,
) -> anyhow::Result<(
	CombinedTimetable,
	HashMap<LessonHashByHomework, HomeworkRaw>,
	HashMap<String, ExamRaw>,
	HashMap<LessonHashByAbsence, AbsenceRaw>,
)> {
	let (timetable, mut homework, mut exams, mut absences) = preprocessed;
	let mut combined = Vec::with_capacity(timetable.len());

	for lesson in timetable {
		let (homework, absence) = match &lesson.subject {
			Some(_subject) => {
				let homework_hash = LessonHashByHomework::new_from_lesson(&lesson);
				let absence_hash = LessonHashByAbsence::new_from_lesson(&lesson);

				let homework = homework.remove(&homework_hash);
				let absence = absences.remove(&absence_hash);

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
	HashMap<LessonHashByHomework, HomeworkRaw>,
	HashMap<String, ExamRaw>,
)> {
	let (timetable, mut homework, mut exams, mut absences) =
		match_preprocessed_internal(preprocessed)?;
	homework.shrink_to_fit();
	exams.shrink_to_fit();
	absences.shrink_to_fit();

	Ok((timetable, homework, exams))
}
