use actix_web::{HttpResponse, Responder, get, web};
use kreta_rs::login::Credentials;
use tokio::sync::Mutex;

use crate::{clients::Clients, timetables::one_month_range};

#[get("/base64/{blob}/timetable.ical")]
pub async fn timetable_base64(
	path: web::Path<String>,
	clients: web::Data<Mutex<Clients>>,
) -> impl Responder {
	let res = async move || {
		let blob = path.into_inner();

		let credentials = crate::k8::decode_base64(&blob)?;
		let timetable = timetable_generic_res(credentials, clients).await?;

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
	credentials: Credentials,
	clients: web::Data<Mutex<Clients>>,
) -> anyhow::Result<String> {
	let res = async move || {
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
