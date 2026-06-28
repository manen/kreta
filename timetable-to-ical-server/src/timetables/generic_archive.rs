use absence_analyzer::retreive::last_september_first;
use actix_web::web;
use chrono::Utc;
use kreta_rs::login::Credentials;
use timetable_to_ical::Options;
use tokio::sync::Mutex;

use crate::clients::Clients;

pub async fn generic_archive_ical(
	credentials: &Credentials,
	clients: web::Data<Mutex<Clients>>,
	opts: &Options,
) -> anyhow::Result<String> {
	let client = {
		let mut clients = clients.lock().await;
		clients.client(credentials).await?
	};
	let client = client.lock().await;

	let (start, end) = (last_september_first(), Utc::now());

	let timetable = {
		timetable_to_ical::combine::combined_range_calendar_file(&client, start, end, opts).await?
	};
	Ok(timetable)
}
