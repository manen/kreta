use crate::AbsencesByExcuse;

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
