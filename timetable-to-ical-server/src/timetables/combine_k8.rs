use actix_web::{HttpResponse, Responder, get, web};
use anyhow::Context;
use credsign::Signer;
use tokio::sync::Mutex;

use crate::clients::Clients;

#[get("/k8/{k8}/combine.ical")]
pub async fn combine_k8(
	k8: web::Path<String>,
	signer: web::Data<Signer>,
	clients: web::Data<Mutex<Clients>>,
) -> impl Responder {
	async fn internal(
		k8: String,
		signer: web::Data<Signer>,
		clients: web::Data<Mutex<Clients>>,
	) -> anyhow::Result<String> {
		let credentials = crate::k8::decode_k8(&k8, &signer)
			.with_context(|| format!("failed to decode k8 {k8}"))?;

		let client = {
			let mut clients = clients.lock().await;
			clients.client(&credentials).await
		}?;

		let timetable = {
			let client = client.lock().await;
			// let (from, to) = super::one_month_range_datetime();
			let (from, to) = super::range_3w_3w();
			let opts = timetable_to_ical::Options::default();
			timetable_to_ical::combine::combined_range_calendar_file(&client, from, to, &opts)
				.await?
		};

		Ok(timetable)
	}

	let res = internal(k8.into_inner(), signer, clients).await;
	let timetable = timetable_to_ical::err::result_as_timetable(res);
	HttpResponse::Ok()
		.content_type("text/calendar")
		.body(timetable)
}

// /// one week back two weeks forward
// fn three_week_range() -> (String, String) {
// 	let today = Utc::now().date_naive();

// 	let start = today - chrono::Duration::days(7);
// 	let end = today + chrono::Duration::days(14);

// 	(
// 		start.format("%Y-%m-%d").to_string(),
// 		end.format("%Y-%m-%d").to_string(),
// 	)
// }
