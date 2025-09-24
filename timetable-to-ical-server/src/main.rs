use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use chrono::Utc;
use kreta_rs::login::Credentials;
use tokio::sync::Mutex;

use crate::clients::Clients;

pub mod clients;

#[get("/")]
async fn index() -> impl Responder {
	"szia"
}

#[get("/basic/{inst_id}/{username}/{passwd}/timetable.ical")]
async fn timetable(
	path: web::Path<(String, String, String)>,
	clients: web::Data<Mutex<Clients>>,
) -> impl Responder {
	let res = async move || {
		let (inst_id, username, passwd) = path.into_inner();
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

	let output_calendar = timetable_to_ical::err::result_as_timetable(res);

	let resp = HttpResponse::Ok()
		.content_type("text/calendar")
		.body(output_calendar);
	resp
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

const BIND: (&str, u16) = ("0.0.0.0", 8080);
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let server = HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(Mutex::new(Clients::default())))
			.service(index)
			.service(timetable)
	})
	.bind(BIND)?
	.run();
	println!("listening on http://{}:{}", BIND.0, BIND.1);

	server.await?;
	Ok(())
}
