use actix_web::{App, HttpServer, web};
use tokio::sync::Mutex;

use crate::clients::Clients;

#[cfg(feature = "combine")]
pub mod combine;

pub mod clients;
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

	let server = HttpServer::new(move || {
		App::new()
			.app_data(clients.clone())
			.app_data(signer.clone())
			.service(landing::index)
			.service(landing::styles)
			.service(timetables::login_base64::timetable_base64)
			.service(timetables::login_k8::timetable_k8)
			.service(timetables::login_k8::combine_k8)
			.service(timetables::login_k8::absences_k8)
			.service(k8::create_k8)
	})
	.bind(BIND)?
	.run();
	println!("listening on http://{}:{}", BIND.0, BIND.1);

	server.await?;
	Ok(())
}
