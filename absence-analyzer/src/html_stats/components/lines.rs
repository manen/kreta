use crate::AbsencesByExcuse;

pub fn hours_total_html(data: &AbsencesByExcuse) -> String {
	let total_hours: f32 = data.absences.iter().map(|a| a.1.hours).sum();
	// let hours_forecasted = crate::forecast::forecast_value(total_hours);

	let body =
		format!("Összesen: <span style=\"font-weight: bold;\"> {total_hours:.1} óra </span>");
	line_common_wrap(&body, 0.2)
}

pub fn unexcused_hours_html(data: &AbsencesByExcuse) -> String {
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

	line_common_wrap(&body, 1.0)
}

fn line_common_wrap(body: &str, half_margin: f32) -> String {
	format!(
		"
		<div style=\"display: flex; align-items: center; flex-direction: column; margin: {half_margin}rem;\">
			<div style=\"margin: {half_margin}rem;\">
				{body}
			</div>
		</div>"
	)
}
