//! contains the different implementations of timetable -> calendar generation

pub mod generic_combine;
pub use generic_combine::generic_combine;
pub mod generic_archive;
pub use generic_archive::generic_archive_ical;

pub mod generic_absences;
pub use generic_absences::generic_absences;

pub mod login_k8;

pub mod utils_receive_opts;
pub use utils_receive_opts::OptsParams;

use chrono::{Local, NaiveDate};

pub(crate) fn range_3w_3w() -> (NaiveDate, NaiveDate) {
	let today = Local::now().date_naive();

	let start = today - chrono::Duration::weeks(3);
	let end = today + chrono::Duration::weeks(3);

	(start, end)
}
