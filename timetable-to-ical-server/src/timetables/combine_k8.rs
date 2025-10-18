use actix_web::{HttpResponse, Responder, get, web};
use anyhow::{Context, anyhow};
use chrono::Utc;
use credsign::Signer;
use kreta_rs::login::Credentials;
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
		let credentials = signer
			.decrypt_text(&k8)
			.with_context(|| "while decrypting k8")?;
		let credentials = {
			let mut credentials = credentials.split('\n').map(String::from);
			let username = credentials
				.next()
				.ok_or_else(|| anyhow!("invalid syntax for k8: first line is username"))?;
			let passwd = credentials
				.next()
				.ok_or_else(|| anyhow!("invalid syntax for k8: second line is password"))?;
			let inst_id = credentials
				.next()
				.ok_or_else(|| anyhow!("invalid syntax for k8: third line is institute id"))?;

			(username, passwd, inst_id)
		};
		let (username, passwd, inst_id) = credentials;

		let credentials = Credentials::new(inst_id, username, passwd);
		let client = {
			let mut clients = clients.lock().await;
			clients.client(&credentials).await
		}?;

		let timetable = {
			let client = client.lock().await;
			let (from, to) = three_week_range();
			let opts = timetable_to_ical::Options::default();
			timetable_to_ical::combine::combined_calendar_file(&client, &from, &to, &opts).await?
		};

		Ok(timetable)
	}

	let res = internal(k8.into_inner(), signer, clients).await;
	let timetable = timetable_to_ical::err::result_as_timetable(res);
	HttpResponse::Ok()
		.content_type("text/calendar")
		.body(timetable)
}

/// one week back two weeks forward
fn three_week_range() -> (String, String) {
	let today = Utc::now().date_naive();

	let start = today - chrono::Duration::days(7);
	let end = today + chrono::Duration::days(14);

	(
		start.format("%Y-%m-%d").to_string(),
		end.format("%Y-%m-%d").to_string(),
	)
}
