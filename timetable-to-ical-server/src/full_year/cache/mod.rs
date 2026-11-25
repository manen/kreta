mod content;

use absence_analyzer::retreive::last_september_first;
use anyhow::Context;
use calhelp::Calendar;
use chrono::NaiveDate;
pub use content::*;
use kreta_rs::{client::Client, login::Credentials};
use safestore::{Db, DbBunch, Encrypted};
use sha2::{Digest, Sha256};
use timetable_to_ical::Options;

fn user_hash(credentials: &Credentials) -> Vec<u8> {
	let mut hasher = Sha256::new();
	hasher.update(credentials.username().as_bytes());
	hasher.update(credentials.passwd().as_bytes());
	hasher.update(credentials.inst_id().as_bytes());
	let hash = hasher.finalize().as_slice().to_owned();
	hash
}

pub struct Caches {
	bunch: DbBunch<Encrypted<CacheContent>>,
}
impl Caches {
	/// hardcoded path from cwd like the .k8 file
	pub async fn new() -> anyhow::Result<Self> {
		let cache_dir_path = std::env::current_dir()
			.with_context(|| "no current dir")?
			.join("timetable_user_cache");

		let bunch = DbBunch::new(&cache_dir_path).await.with_context(|| {
			format!(
				"failed to create DbBunch with dir path {}",
				cache_dir_path.display()
			)
		})?;

		Ok(Self { bunch })
	}

	pub async fn db_for_user(
		&self,
		credentials: &Credentials,
	) -> anyhow::Result<Db<Encrypted<CacheContent>>> {
		let user_hash = user_hash(&credentials);
		let db = self.bunch.db(&user_hash).await.with_context(|| {
			format!(
				"failed to open full year cache for user {} (user hash {user_hash:?})",
				credentials.username()
			)
		})?;

		Ok(db)
	}
}

pub async fn assemble_full_year_from_encrypted_cache(
	cache: &mut Encrypted<CacheContent>,
	fresh_from: NaiveDate,
	fresh_to: NaiveDate,
	credentials: &Credentials,
	client: &Client,
	opts: &Options,
	fresh_update_if_older: chrono::Duration,
	stale_update_if_older: chrono::Duration,
) -> anyhow::Result<Calendar> {
	let calendar = cache
		.with_mut_async(credentials, async |cache| {
			assemble_full_year_from_cache(
				cache,
				fresh_from,
				fresh_to,
				client,
				opts,
				fresh_update_if_older,
				stale_update_if_older,
			)
			.await
		})
		.await
		.with_context(|| {
			format!(
				"failed to mutate Encrypted<CacheContent> for user {}",
				credentials.username()
			)
		})?;

	Ok(calendar)
}

pub async fn assemble_full_year_from_cache(
	cache: &mut CacheContent,
	fresh_from: NaiveDate,
	fresh_to: NaiveDate,
	client: &Client,
	opts: &Options,
	fresh_update_if_older: chrono::Duration,
	stale_update_if_older: chrono::Duration,
) -> Calendar {
	let mut refresh = async || {
		let last_september_first = last_september_first();
		let next_september_first = last_september_first + chrono::Duration::days(365);

		let did_refresh_everything = cache
			.refresh_days_conditional(
				last_september_first,
				next_september_first,
				client,
				opts,
				stale_update_if_older,
			)
			.await?;

		if !did_refresh_everything {
			// no need to refresh 3w-3w if everything was just requested
			cache
				.refresh_days_conditional(fresh_from, fresh_to, client, opts, fresh_update_if_older)
				.await?;
		}
		cache.discard_if_older_than(last_september_first);

		anyhow::Ok(())
	};
	let refresh = refresh().await;

	let mut calendar = cache.to_calendar();

	match refresh {
		Ok(a) => a,
		Err(err) => {
			let err_event = timetable_to_ical::err::error_as_event(err);
			let err_calendar = Calendar::new(std::iter::once(err_event));
			calendar = calendar.merge(err_calendar);
		}
	}

	calendar
}
pub async fn assemble_full_year_from_db(
	db: &mut Db<Encrypted<CacheContent>>,
	fresh_from: NaiveDate,
	fresh_to: NaiveDate,
	credentials: &Credentials,
	client: &Client,
	opts: &Options,
	fresh_update_if_older: chrono::Duration,
	stale_update_if_older: chrono::Duration,
) -> anyhow::Result<Calendar> {
	let calendar = db
		.with_mut(async |cache| {
			assemble_full_year_from_encrypted_cache(
				cache,
				fresh_from,
				fresh_to,
				credentials,
				client,
				opts,
				fresh_update_if_older,
				stale_update_if_older,
			)
			.await
		})
		.await
		.with_context(|| format!("while saving updated cache"))?
		.with_context(|| format!("while refreshing cache"))?;
	Ok(calendar)
}
