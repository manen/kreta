use actix_web::{HttpResponse, Responder, get, web};
use anyhow::{Context, anyhow};
use base64::Engine;
use chrono::Utc;
use kreta_rs::login::Credentials;
use tokio::sync::Mutex;

use crate::clients::Clients;

#[get("/base64/{blob}/timetable.ical")]
pub async fn timetable_base64(
	path: web::Path<String>,
	clients: web::Data<Mutex<Clients>>,
) -> impl Responder {
	let res = async move || {
		let blob = path.into_inner();
		let blob = base64::prelude::BASE64_URL_SAFE
			.decode(&blob)
			.with_context(|| "while decoding the base64 blob provided")?;
		let blob = String::from_utf8(blob).with_context(|| "base64 encoded blob is not utf-8")?;

		let mut credentials = blob.split('\n').map(String::from);
		let username = credentials
			.next()
			.ok_or_else(|| anyhow!("invalid syntax for the base64 blob: first line is username"))?;
		let passwd = credentials.next().ok_or_else(|| {
			anyhow!("invalid syntax for the base64 blob: second line is password")
		})?;
		let inst_id = credentials.next().ok_or_else(|| {
			anyhow!("invalid syntax for the base64 blob: third line is institute id")
		})?;

		let timetable = timetable_generic_res((inst_id, username, passwd), clients).await?;

		anyhow::Ok(timetable)
	};
	let res = res().await;

	let output_calendar = timetable_to_ical::err::result_as_timetable(res);

	let resp = HttpResponse::Ok()
		.content_type("text/calendar")
		.body(output_calendar);
	resp
}

async fn timetable_generic_res(
	(inst_id, username, passwd): (String, String, String),
	clients: web::Data<Mutex<Clients>>,
) -> anyhow::Result<String> {
	let res = async move || {
		let credentials = Credentials::new(inst_id, username, passwd);

		let client = {
			let mut clients = clients.lock().await;
			clients.client(&credentials).await?
		};
		let client = client.lock().await;

		let (start, end) = one_month_range();
		let timetable = client.timetable(&start, &end).await?;
		let timetable =
			timetable_to_ical::lessons_to_calendar_file(&timetable, &Default::default());

		anyhow::Ok(timetable)
	};
	let res = res().await;
	res
}

/// one month range centered on today
fn one_month_range() -> (String, String) {
	let today = Utc::now().date_naive();

	let start = today - chrono::Duration::days(14);
	let end = today + chrono::Duration::days(14);

	(
		start.format("%Y-%m-%d").to_string(),
		end.format("%Y-%m-%d").to_string(),
	)
}
