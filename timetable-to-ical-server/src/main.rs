use actix_web::{App, HttpServer, web};
use anyhow::Context;
use tokio::sync::Mutex;

use crate::{clients::Clients, full_year::cache::Caches};

pub mod combine;

pub mod clients;
pub mod full_year;
pub mod k8;
pub mod landing;
pub mod timetables;

const BIND: (&str, u16) = const {
	let port = if cfg!(debug_assertions) { 8080 } else { 18080 };
	("0.0.0.0", port)
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let signer = credsign::load_or_create(std::env::current_dir()?).await?;
	let signer = web::Data::new(signer);

	let clients = web::Data::new(Mutex::new(Clients::default()));
	let caches = Caches::new()
		.await
		.with_context(|| format!("failed to create common Caches cache manager"))?;
	let caches = web::Data::new(Mutex::new(caches));

	let server = HttpServer::new(move || {
		App::new()
			.app_data(clients.clone())
			.app_data(caches.clone())
			.app_data(signer.clone())
			.service(landing::index)
			.service(landing::styles)
			.service(timetables::login_k8::combine_k8)
			.service(timetables::login_k8::full_year_k8)
			.service(timetables::login_k8::absences_k8)
			.service(timetables::login_k8::archive_ical_k8)
			.service(k8::create_k8)
	})
	.bind(BIND)?
	.run();
	println!("listening on http://{}:{}", BIND.0, BIND.1);

	server.await?;
	Ok(())
}
