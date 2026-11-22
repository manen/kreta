use std::collections::HashMap;

use anyhow::Context;
use calhelp::{Calendar, Event};
use chrono::{Local, NaiveDate};
use kreta_rs::client::Client;
use serde::{Deserialize, Serialize};
use timetable_to_ical::Options;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

		let now = Local::now();
		let now_fmt = now.format("%Y-%m-%d %H:%M:%S");
		let last_updated_line = format!("\n\nlast updated {now_fmt}");

		let events = events.filter_map(|mut event| {
			// append last updated date to desc
			event.description(&last_updated_line);

			let day = event.get_dt_start()?.date_naive();
			Some((day, event))
		});

		let mut days_append_buf = HashMap::<NaiveDate, CacheDayData>::new();
		// add all the events we have
		for (date, event) in events {
			match days_append_buf.get_mut(&date) {
				Some(cache_day_data) => cache_day_data.events.push(event),
				None => {
					// create new cachedaydata
					let cache_day_data = CacheDayData {
						last_updated: now,
						events: vec![event],
					};
					days_append_buf.insert(date, cache_day_data);
				}
			}
		}
		// for the days we requested but didn't receive any events for we still have to make a CacheDayData
		// so we know we have data for that day it's just no events
		{
			let mut i = from;
			while i <= to {
				if !days_append_buf.contains_key(&i) {
					let cache_day_data = CacheDayData {
						last_updated: now,
						events: vec![],
					};
					days_append_buf.insert(i, cache_day_data);
				}
				i += chrono::Duration::days(1);
			}
		}

		// create a new updated hashmap and put it in self.days
		self.days = self
			.days
			.drain()
			.chain(days_append_buf.into_iter())
			.collect();

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

		let now = Local::now();

		let mut should_fetch_fresh = false;
		let mut i = from;
		while i <= to && !should_fetch_fresh {
			let day = self.days.get(&i);
			match day {
				Some(day) => {
					let last_updated_elapsed = now - day.last_updated;
					// println!(
					// 	"last updated: {last_updated_elapsed}\nupdate if older: {update_if_older}\n"
					// );
					if last_updated_elapsed > update_if_older {
						should_fetch_fresh = true;
					}
				}
				None => {
					should_fetch_fresh = true;
				}
			}

			i += chrono::Duration::days(1);
		}

		if should_fetch_fresh {
			println!(
				"cache is fetching from {from} to {to} since something was older than {update_if_older}"
			);
			self.request_days_fresh(from, to, client, opts).await
		} else {
			println!(
				"cache is returning old data from {from} to {to} since nothing is older than {update_if_older}"
			);
			Ok(())
		}
	}

	// delete
	pub fn discard_if_older_than(&mut self, oldest_to_keep: NaiveDate) {
		self.days.extract_if(|day, _| *day < oldest_to_keep).count();
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheDayData {
	last_updated: chrono::DateTime<Local>,
	events: Vec<Event>,
}
