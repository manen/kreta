use actix_web::web;
use kreta_rs::login::Credentials;
use timetable_to_ical::Options;
use tokio::sync::Mutex;

use crate::{clients::Clients, timetables::one_month_range};

/// basic timetable implentation for generic, single query timetable requests
pub async fn generic_timetable(
	credentials: &Credentials,
	clients: web::Data<Mutex<Clients>>,
) -> anyhow::Result<String> {
	let client = {
		let mut clients = clients.lock().await;
		clients.client(credentials).await?
	};
	let client = client.lock().await;

	let (start, end) = one_month_range();
	let timetable = client.timetable(&start, &end).await?;

	let opts = Options::default();
	let timetable = timetable_to_ical::lessons_to_calendar_file(&timetable, &opts);

	anyhow::Ok(timetable)
}
