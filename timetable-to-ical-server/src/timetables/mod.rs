//! contains the different implementations of timetable -> calendar generation

pub mod generic_timetable;
pub use generic_timetable::generic_timetable;
pub mod generic_combine;
pub use generic_combine::generic_combine;

pub mod login_base64;
pub mod login_k8;

use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};

/// one month range centered on today
fn one_month_range_base() -> (chrono::NaiveDate, chrono::NaiveDate) {
	let today = Utc::now().date_naive();

	let start = today - chrono::Duration::days(14);
	let end = today + chrono::Duration::days(14);

	(start, end)
}

fn one_month_range() -> (String, String) {
	let (start, end) = one_month_range_base();
	(
		start.format("%Y-%m-%d").to_string(),
		end.format("%Y-%m-%d").to_string(),
	)
}

fn one_month_range_datetime() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
	let (start, end) = one_month_range_base();
	let time = NaiveTime::from_hms_opt(0, 0, 0).expect("from_hms_opt(0, 0, 0) failed");

	let start = NaiveDateTime::new(start, time);
	let end = NaiveDateTime::new(end, time);

	let start = DateTime::from_naive_utc_and_offset(start, Utc);
	let end = DateTime::from_naive_utc_and_offset(end, Utc);

	(start, end)
}

fn two_month_range_datetime() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
	let (start, end) = {
		let today = Utc::now().date_naive();
		let start = today - chrono::Duration::days(30);
		let end = today + chrono::Duration::days(30);
		(start, end)
	};
	let time = NaiveTime::from_hms_opt(0, 0, 0).expect("from_hms_opt(0, 0, 0) failed");

	let start = NaiveDateTime::new(start, time);
	let end = NaiveDateTime::new(end, time);

	let start = DateTime::from_naive_utc_and_offset(start, Utc);
	let end = DateTime::from_naive_utc_and_offset(end, Utc);

	(start, end)
}

fn range_3w_3w() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>) {
	let (start, end) = {
		let today = Utc::now().date_naive();
		let start = today - chrono::Duration::weeks(3);
		let end = today + chrono::Duration::weeks(3);
		(start, end)
	};
	let time = NaiveTime::from_hms_opt(0, 0, 0).expect("from_hms_opt(0, 0, 0) failed");

	let start = NaiveDateTime::new(start, time);
	let end = NaiveDateTime::new(end, time);

	let start = DateTime::from_naive_utc_and_offset(start, Utc);
	let end = DateTime::from_naive_utc_and_offset(end, Utc);

	(start, end)
}
