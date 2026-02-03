#![allow(unused)]

use chrono::TimeZone;
use kreta_rs::{client::Client, login::LoginFlow};

mod creds_from_file;

fn main() {
	println!("Hello, world!");

	let rt = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap();

	rt.block_on(absences()).unwrap()
}

#[allow(deprecated)]
fn parse_simple_date(simple: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
	let date = chrono::NaiveDate::parse_from_str(simple, "%Y-%m-%d")?;
	let datetime = chrono::Utc.from_utc_date(&date).and_hms(0, 0, 0);

	Ok(datetime)
}

async fn absences() -> anyhow::Result<()> {
	let credentials = creds_from_file::read_from_file("./credentials.txt").await?;

	let mut client = Client::full_login(&credentials).await?;

	let (from, to) = ("2025-12-13", "2026-02-03");
	let (from, to) = (parse_simple_date(from)?, parse_simple_date(to)?);

	let absences = client.absences_range(from, to).await?;
	println!("{absences:#?}");

	Ok(())
}

async fn homework() -> anyhow::Result<()> {
	let credentials = creds_from_file::read_from_file("./credentials.txt").await?;

	let mut client = Client::full_login(&credentials).await?;

	// let hw = client.homework("2025-09-26", "2025-10-17").await?;

	let hw = client.exams("2025-09-26", "2025-10-17").await?;
	println!("{hw:?}");

	Ok(())
}

async fn start() -> anyhow::Result<()> {
	let credentials = creds_from_file::read_from_file("./credentials.txt").await?;

	let mut client = Client::full_login(&credentials).await?;
	tokio::time::sleep(std::time::Duration::from_secs(5)).await;

	client.refresh().await?;
	println!("refreshing works");

	Ok(())
}

async fn execute_login_flow() -> anyhow::Result<()> {
	let credentials = creds_from_file::read_from_file("./credentials.txt").await?;

	let login_flow = LoginFlow::new()?;

	let data = login_flow.begin().await?;
	login_flow.post_credentials(&data, &credentials).await?;
	let tokens = login_flow.request_token(&data).await?;

	println!("{tokens:#?}");

	Ok(())
}

async fn gen_timetable() -> anyhow::Result<()> {
	let credentials = creds_from_file::read_from_file("./credentials.txt").await?;
	let client = Client::full_login(&credentials).await?;

	let opts = timetable_to_ical::Options {
		..Default::default()
	};

	let timetable = client.timetable("2025-10-21", "2025-11-04").await?;
	let calendar = timetable_to_ical::lessons_to_calendar_file(&timetable, &opts);

	tokio::fs::write("./timetable.ical", &calendar).await?;

	println!("all done");
	Ok(())
}
