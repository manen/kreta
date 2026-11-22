use actix_web::web;
use anyhow::Context;
use kreta_rs::login::Credentials;
use timetable_to_ical::Options;
use tokio::sync::Mutex;

use crate::{clients::Clients, full_year::cache::Caches, timetables::range_3w_3w};

pub mod cache;

pub async fn full_year_assemble_ical(
	credentials: &Credentials,
	clients: web::Data<Mutex<Clients>>,
	caches: web::Data<Mutex<Caches>>,
	opts: &Options,
) -> anyhow::Result<String> {
	let client = {
		let mut clients = clients.lock().await;
		clients.client(credentials).await?
	};
	let client = client.lock().await;

	println!("serving full year for {}", credentials.username());
	let mut cache_db = {
		let caches = caches.lock().await;
		caches.db_for_user(credentials).await?
	};

	let (fresh_from, fresh_to) = range_3w_3w();
	let fresh_update_if_older = chrono::Duration::minutes(10);
	let stale_update_if_older = chrono::Duration::days(1);

	let calendar = cache::assemble_full_year_from_db(
		&mut cache_db,
		fresh_from,
		fresh_to,
		credentials,
		&client,
		opts,
		fresh_update_if_older,
		stale_update_if_older,
	)
	.await
	.with_context(|| {
		format!(
			"while assembling full year calendar for user {}",
			credentials.username()
		)
	})?;

	Ok(calendar.to_string_publish())
}
