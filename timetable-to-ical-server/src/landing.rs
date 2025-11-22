use actix_web::{HttpResponse, Responder, get, web};
use anyhow::Context;
use timetable_to_ical::Options;

const LANDING_STATIC: &str = include_str!("../static/landing.html");
const STYLES_STATIC: &str = include_str!("../static/styles.css");

#[get("/")]
async fn index() -> impl Responder {
	let default_opts = Options::default();
	let default_opts_json = serde_json::to_string_pretty(&default_opts)
		.with_context(|| format!("failed to stringify default timetable-to-ical options"));

	let default_opts_json = match default_opts_json {
		Ok(a) => a,
		Err(err) => {
			eprintln!("failed to format options: {err}\n\n{err:?}");
			"failed to format options: {err}\nif you don't edit this text it should still work fine tho"
				.into()
		}
	};
	let landing = LANDING_STATIC.replace("{%default_options%}", &default_opts_json);

	web::Html::new(landing)
}

#[get("/styles.css")]
async fn styles() -> impl Responder {
	HttpResponse::Ok()
		.content_type("text/css")
		.body(STYLES_STATIC)
}
