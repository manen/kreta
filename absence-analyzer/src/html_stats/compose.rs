use kreta_rs::client::absences::AbsenceRaw;

use crate::html_stats::components;

const FRAME: &'static str = include_str!("./frame.html");
pub fn html_stats(data: &[AbsenceRaw], today: chrono::DateTime<chrono::Utc>) -> String {
	let content = html_stats_content(data, today);
	FRAME.replace("{content}", &content)
}

pub fn html_stats_content(
	absences_raw: &[AbsenceRaw],
	today: chrono::DateTime<chrono::Utc>,
) -> String {
	let by_excuse = crate::absences_by_excuse_type(absences_raw);

	let graph = components::by_excuse_type(&by_excuse);
	let forecast = components::forecast_html(&by_excuse);
	let line_graph = components::line_graph(absences_raw.iter().cloned(), today);

	format!(
		"
		{graph}
		\n\n{forecast}
		\n\n{line_graph}"
	)
}
