use actix_web::web;
use kreta_rs::login::Credentials;
use timetable_to_ical::Options;
use tokio::sync::Mutex;

use crate::{clients::Clients, timetables::range_3w_3w};

/// generic implementation for the timetable variation that combines timetable, exams and homeworks to build the timetable
pub async fn generic_combine(
	credentials: &Credentials,
	clients: web::Data<Mutex<Clients>>,
) -> anyhow::Result<String> {
	let client = {
		let mut clients = clients.lock().await;
		clients.client(credentials).await?
	};
	let client = client.lock().await;

	let (start, end) = range_3w_3w();
	let opts = Options::default();
	let timetable =
		timetable_to_ical::combine::combined_range_calendar_file(&client, start, end, &opts)
			.await?;

	Ok(timetable)
}
