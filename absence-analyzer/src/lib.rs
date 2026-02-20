pub mod by_excuse;
pub mod by_week;
pub mod forecast;
pub mod html_stats;
pub mod retreive;

pub use by_excuse::*;
pub use by_week::split_by_week;
pub use forecast::extract_unexcused_forecast;
pub use html_stats::html_stats;
pub use retreive::fetch_absences;
