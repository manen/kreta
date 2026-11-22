mod content;
use std::hash::{DefaultHasher, Hash, Hasher};

use absence_analyzer::retreive::last_september_first;
use anyhow::Context;
use calhelp::Calendar;
use chrono::NaiveDate;
pub use content::*;
use kreta_rs::{client::Client, login::Credentials};
use safestore::{Db, DbBunch};
use timetable_to_ical::Options;

// all is left is to create this caches and make it a web::Data variable
// and have the generic access it and call this function and we should all be done

fn user_hash(credentials: &Credentials) -> u64 {
	let mut hasher = DefaultHasher::default();
	credentials.hash(&mut hasher);
	hasher.finish()
}

pub struct Caches {
	bunch: DbBunch<CacheContent>,
}
impl Caches {
	/// hardcoded path from cwd like the .k8 file
	pub async fn new() -> anyhow::Result<Self> {
		let cache_dir_path = std::env::current_dir()
			.with_context(|| "no current dir")?
			.join("timetable_user_cache");

		let bunch = DbBunch::new(&cache_dir_path).with_context(|| {
			format!(
				"failed to create DbBunch with dir path {}",
				cache_dir_path.display()
			)
		})?;

		Ok(Self { bunch })
	}

	pub async fn db_for_user(&self, credentials: &Credentials) -> anyhow::Result<Db<CacheContent>> {
		let user_hash = user_hash(&credentials);
		let entry_id = user_hash.to_le_bytes();
		let db = self
			.bunch
			.db(&entry_id)
			.with_context(|| format!("failed to open full year cache for user hash {user_hash}"))?;

		Ok(db)
	}
}

pub async fn assemble_full_year_from_cache(
	cache: &mut CacheContent,
	fresh_from: NaiveDate,
	fresh_to: NaiveDate,
	client: &Client,
	opts: &Options,
	stale_update_if_older: chrono::Duration,
) -> anyhow::Result<Calendar> {
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
}
pub async fn assemble_full_year_from_db(
	db: &mut Db<CacheContent>,
	fresh_from: NaiveDate,
	fresh_to: NaiveDate,
	client: &Client,
	opts: &Options,
	stale_update_if_older: chrono::Duration,
) -> anyhow::Result<Calendar> {
	let calendar = db
		.with_mut(async |cache| {
			assemble_full_year_from_cache(
				cache,
				fresh_from,
				fresh_to,
				client,
				opts,
				stale_update_if_older,
			)
			.await
		})
		.await
		.with_context(|| format!("while saving updated cache"))?
		.with_context(|| format!("while refreshing cache"))?;
	Ok(calendar)
}
