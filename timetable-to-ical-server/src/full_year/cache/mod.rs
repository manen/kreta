mod content;
use absence_analyzer::retreive::last_september_first;
use anyhow::Context;
use calhelp::Calendar;
use chrono::NaiveDate;
pub use content::*;
use kreta_rs::client::Client;
use safestore::DbBunch;
use timetable_to_ical::Options;

// all is left is to create this caches and make it a web::Data variable
// and have the generic access it and call this function and we should all be done

pub struct Caches {
	bunch: DbBunch<CacheContent>,
}
impl Caches {
	pub async fn assemble_full_year(
		&mut self,
		user_hash: u64,
		fresh_from: NaiveDate,
		fresh_to: NaiveDate,
		client: &Client,
		opts: &Options,
		stale_update_if_older: chrono::Duration,
	) -> anyhow::Result<Calendar> {
		let entry_id = user_hash.to_le_bytes();
		let mut db = self
			.bunch
			.db(&entry_id)
			.with_context(|| format!("failed to open full year cache for user hash {user_hash}"))?;

		let calendar = db
			.with_mut(async |cache| {
				let last_september_first = last_september_first();
				let next_september_first = last_september_first + chrono::Duration::days(365);

				cache
					.request_days_fresh(fresh_from, fresh_to, client, opts)
					.await?;
				cache
					.refresh_days_conditional(
						last_september_first,
						next_september_first,
						client,
						opts,
						stale_update_if_older,
					)
					.await?;

				let calendar = cache.to_calendar();
				anyhow::Ok(calendar)
			})
			.await
			.with_context(|| format!("while saving updated cache for user hash {user_hash}"))?
			.with_context(|| format!("while refreshing cache for user hash {user_hash}"))?;
		Ok(calendar)
	}
}
