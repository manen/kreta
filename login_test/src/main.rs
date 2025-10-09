#![allow(unused)]

use kreta_rs::{client::Client, login::LoginFlow};

mod creds_from_file;

fn main() {
	println!("Hello, world!");

	let rt = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap();

	rt.block_on(gen_timetable()).unwrap()
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

	let timetable = client.timetable("2025-09-25", "2025-10-23").await?;
	let calendar = timetable_to_ical::lessons_to_calendar_file(&timetable, &opts);

	tokio::fs::write("./timetable.ical", &calendar).await?;

	println!("all done");
	Ok(())
}
