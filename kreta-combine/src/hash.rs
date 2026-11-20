use std::hash::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::{NonZero, NonZeroU64};

use kreta_rs::types::{BriefLessonRaw, HomeworkRaw, LessonRaw};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LessonHashByAbsence(u64);
impl LessonHashByAbsence {
	pub fn new_from_absence(brief_lesson: &BriefLessonRaw) -> Self {
		let mut hasher = DefaultHasher::new();

		println!("registering absence with brief lesson {brief_lesson:?}");
		brief_lesson.start_time.hash(&mut hasher);
		brief_lesson.end_time.hash(&mut hasher);

		Self(hasher.finish())
	}
	pub fn new_from_lesson(lesson: &LessonRaw) -> Self {
		let mut hasher = DefaultHasher::new();

		lesson.start_time.hash(&mut hasher);
		lesson.end_time.hash(&mut hasher);

		Self(hasher.finish())
	}
}

// najo az a helyzet hogy a combinet szetszedtem es ezt a hashelest kene megcsinalni
// elvileg most minden logikusan van layoutolva es abbol hogy hol van error es itt
// mit kezdtem el abbol meg lehet erteni h mi volt az elkepzeles
//

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// apparently homework already has a uid we can use so this is simpler
pub struct LessonHashByHomework(Option<NonZero<u64>>);

// this impl is faulty because it attaches to the lesson it was given out not the deadline lesson
impl LessonHashByHomework {
	pub fn new_from_homework(homework: &HomeworkRaw) -> Self {
		let mut hasher = DefaultHasher::new();

		homework.uid.hash(&mut hasher);

		Self(NonZeroU64::new(hasher.finish()))
	}
	pub fn new_from_lesson(lesson: &LessonRaw) -> Self {
		let mut hasher = DefaultHasher::new();

		match &lesson.homework_uid {
			Some(uid) => {
				uid.hash(&mut hasher);
				Self(NonZeroU64::new(hasher.finish()))
			}
			None => Self(None),
		}
	}
}

// impl LessonHashByHomework {
// 	pub fn new_from_homework(homework: &HomeworkRaw) -> Self {
// 		let mut hasher = DefaultHasher::new();
//
// 		// jo ahhoz hogy ezt normalisan megcsinaljam meg kene csinalni hogy a kretas adat strukturakban ne string legyen az ido hanem proper chrono datetime
// 		// ami elvileg tud ilyet serdevel
// 		//
// 		// de ez kurva nagy valtozas lenne es ezt nem este 10kor fogom elkezdeni ugy hogy meg millioegy dolgom van
// 		//
// 		// de utana sokkal egyszerubb lenne minden es itt is pl sokkal konnyebben megnezhetnenk hogy aznapi ora legyen stb stb stb stb joejt
// 	}
// 	pub fn new_from_lesson(lesson: &LessonRaw) -> Self {}
// }

// elvileg mukodik
// de se az examokat se a homeworkoket nem tudom most tesztelni
// mert nem irtak be egy darabot se
// szoval bleehhhh ezt most itt kell hagynom a working directoryba de stagelem vagy commitolom legalabb

// jo beirtak az elso dogat az mukodik de az nincs is benne ebben a hashes systemben
// az elso hazinal visszaterek
