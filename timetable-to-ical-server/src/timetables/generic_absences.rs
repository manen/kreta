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
	#[cfg(feature = "absence-analyzer")]
	{
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

		let html = absence_analyzer::html_stats::html_stats(&absences_raw);

		Ok(Html::new(html))
	}

	#[cfg(not(feature = "absence-analyzer"))]
	{
		Ok(Html::new(
			"absence-analyzer wasn't enabled on this server at compile time",
		))
	}
}
