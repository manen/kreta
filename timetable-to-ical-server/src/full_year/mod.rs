use actix_web::web;
use kreta_rs::login::Credentials;
use timetable_to_ical::Options;
use tokio::sync::Mutex;

use crate::clients::Clients;

pub mod cache;

pub async fn full_year_assemble_ical(
	credentials: &Credentials,
	clients: web::Data<Mutex<Clients>>,
	opts: &Options,
) -> anyhow::Result<String> {
	let client = {
		let mut clients = clients.lock().await;
		clients.client(credentials).await?
	};
	let client = client.lock().await;

	// amugy konkretan a legvegen vagyok
	// mar van store meg van minden
	// most a dbt kell bekotni ide hogy itt betoltsuk ennek a usernek a dbjet es megnezzuk hogy van e benne cache
	// es ha van akkor nem-e tul regi
	//
	// a legutobbi szeptembertol 3 hettel ezelottig cachben turkalunk
	// 3 hettel ezelottol 3 hettel ezutanig fresh request mindig
	// 3 hettel ezutantol julius 15eig cachben turkalunk

	// let (fresh_start, fresh_end) = crate::timetables::range_3w_3w();
	// let fresh =
	// 	timetable_to_ical::combine::combined_range_calendar(&client, start, end, opts).await?;

	Ok(())
}
