use actix_web::web::{self, Html};
use anyhow::Context;
use kreta_rs::login::Credentials;
use tokio::sync::Mutex;

use crate::clients::Clients;

/// not really a timetable implementation this returns html
pub async fn generic_absences(
	credentials: &Credentials,
	clients: web::Data<Mutex<Clients>>,
) -> anyhow::Result<Html> {
	let absences_raw = {
		let client = {
			let mut clients = clients.lock().await;
			clients.client(credentials).await?
		};
		let client = client.lock().await;

		let absences = absence_analyzer::retreive::fetch_absences(&client)
			.await
			.with_context(|| {
				format!(
					"failed to fetch every absence for user {}",
					credentials.username()
				)
			})?;

		absences
	};

	let absences = absence_analyzer::absences_by_excuse_type(absences_raw.iter());
	let html = absence_analyzer::html_stats::html_stats(&absences);

	Ok(Html::new(html))
}
