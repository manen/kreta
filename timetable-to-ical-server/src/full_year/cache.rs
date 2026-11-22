use std::collections::HashMap;

use anyhow::Context;
use calhelp::{Calendar, Event};
use chrono::{Local, NaiveDate};
use kreta_rs::client::Client;
use timetable_to_ical::Options;

pub struct Cache {}

#[derive(Debug, Clone, Default)]
pub struct CacheContent {
	days: HashMap<NaiveDate, CacheDayData>,
}
impl CacheContent {
	// create
	pub fn new() -> Self {
		Self::default()
	}

	// update
	pub async fn request_days_fresh(
		&mut self,
		from: NaiveDate,
		to: NaiveDate,
		client: &Client,
		opts: &Options,
	) -> anyhow::Result<()> {
		let timetable = timetable_to_ical::combine::combined_range_calendar(client, from, to, opts)
			.await
			.with_context(|| format!("while refreshing cache from {from} to {to}"))?;

		let events = timetable.take_events();
		// append last updated date to desc
		let now = Local::now().date_naive();
		let last_updated_line = format!("\n\nlast updated {now}");

		let events = events.map(|mut event| {
			event.description(&last_updated_line);

			let day = event.dt_start(dt_start)
			
			event
		});

		Ok(())
	}
	pub async fn refresh_days_conditional(
		&mut self,
		from: NaiveDate,
		to: NaiveDate,
		client: &Client,
		opts: &Options,
		update_if_older: chrono::Duration,
	) -> anyhow::Result<()> {
		// we're making a shortcut and only not fetching again if all the days are in cache limits
		// so if even a day's too old we're refetching everything

		let elapsed = Local::now().date_naive().signed_duration_since(to);
		let should_fetch_fresh = elapsed > update_if_older;

		if should_fetch_fresh {
			self.request_days_fresh(from, to, client, opts).await
		} else {
			Ok(())
		}
	}

	// delete
	pub fn discard_if_older_than(&mut self, oldest_to_keep: NaiveDate) {
		self.days.extract_if(|day, _| *day < oldest_to_keep);
	}

	// read
	pub fn to_calendar(&self) -> Calendar {
		Calendar::new(
			self.days
				.iter()
				.map(|(_, data)| data.events.iter())
				.flatten()
				.cloned(),
		)
	}
}

#[derive(Debug, Clone)]
pub struct CacheDayData {
	last_updated: chrono::DateTime<Local>,
	events: Vec<Event>,
}
