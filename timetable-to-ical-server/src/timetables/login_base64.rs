use actix_web::{HttpResponse, Responder, get, web};
use anyhow::Context;
use timetable_to_ical::err::handle_timetable_err_async;
use tokio::sync::Mutex;

use crate::{
	clients::Clients,
	timetables::{OptsParams, generic_timetable},
};

#[get("/base64/{blob}/timetable.ical")]
pub async fn timetable_base64(
	path: web::Path<String>,
	clients: web::Data<Mutex<Clients>>,
	opts: web::Query<OptsParams>,
) -> impl Responder {
	let timetable = handle_timetable_err_async(async move {
		let base64 = path.into_inner();
		let credentials = crate::k8::decode_base64(&base64)
			.with_context(|| format!("failed to decode base64 {base64}"))?;

		let opts = opts.extract_options()?;

		let timetable = generic_timetable(&credentials, clients, &opts).await?;
		anyhow::Ok(timetable)
	})
	.await;

	HttpResponse::Ok()
		.content_type("text/calendar")
		.body(timetable)
}
