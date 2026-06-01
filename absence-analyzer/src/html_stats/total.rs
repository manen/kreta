use kreta_rs::client::absences::AbsenceRaw;

use crate::{AbsencesByExcuse, absences_by_excuse_type, by_week::split_by_week_and_excuse};

const FRAME: &'static str = include_str!("./frame.html");
pub fn html_stats(data: &[AbsenceRaw], today: chrono::DateTime<chrono::Utc>) -> String {
	let content = html_stats_content(data, today);
	FRAME.replace("{content}", &content)
}

pub fn html_stats_content(
	absences_raw: &[AbsenceRaw],
	today: chrono::DateTime<chrono::Utc>,
) -> String {
	let by_excuse = absences_by_excuse_type(absences_raw);

	let graph = super::by_excuse_type(&by_excuse);
	let forecast = forecast_html(&by_excuse);

	let line_graph = line_graph(absences_raw.iter().cloned(), today);

	format!(
		"
		{graph}
		\n\n{forecast}
		\n\n{line_graph}"
	)
}

pub fn line_graph(
	absences_raw: impl Iterator<Item = AbsenceRaw>,
	today: chrono::DateTime<chrono::Utc>,
) -> String {
	let by_week = split_by_week_and_excuse(absences_raw);
	let line_graph = by_week.map(|a| super::line_by_week(&a, today));

	line_graph.unwrap_or_else(|err| {
		format!(
			"could not create line graph due to an error while splitting up absences by week:\n{err}"
		)
	})
}

pub fn forecast_html(data: &AbsencesByExcuse) -> String {
	let forecast = crate::forecast::extract_unexcused_forecast(data);

	let body = match forecast {
		None => "Nincs igazolatlan mulasztásod.".to_string(),

		Some(forecast) if forecast.only_unexcused == forecast.with_to_be_excused => {
			format!(
				"Év végére kb. <span style=\"font-weight: bold;\">{:.1} óra</span> igazolatlanod lesz.",
				forecast.only_unexcused
			)
		}
		Some(forecast) => {
			format!(
				"
				Év végére kb. <span style=\"font-weight: bold;\">{:.1} óra</span> igazolatlanod lesz. <br>
				Ha az igazolandó mulasztásaid igazolva lesznek, ez csak <span style=\"font-weight: bold;\">{:.1} óra</span>.
				",
				forecast.with_to_be_excused, forecast.only_unexcused
			)
		}
	};
	format!(
		"
		<div style=\"display: flex; align-items: center; flex-direction: column; margin: 1rem;\">
			<div style=\"margin: 1rem;\">
				{body}
			</div>
		</div>"
	)
}
