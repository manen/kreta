use actix_web::{HttpResponse, Responder, get, web};
use anyhow::Context;
use credsign::Signer;
use timetable_to_ical::err::handle_timetable_err_async;
use tokio::sync::Mutex;

use crate::clients::Clients;

#[get("/k8/{k8}/combine.ical")]
pub async fn combine_k8(
	k8: web::Path<String>,
	signer: web::Data<Signer>,
	clients: web::Data<Mutex<Clients>>,
) -> impl Responder {
	let timetable = handle_timetable_err_async(async move {
		let k8 = k8.into_inner();
		let credentials = crate::k8::decode_k8(&k8, &signer)
			.with_context(|| format!("failed to decode k8 {k8}"))?;

		let timetable = super::generic_combine(&credentials, clients).await?;
		anyhow::Ok(timetable)
	})
	.await;

	HttpResponse::Ok()
		.content_type("text/calendar")
		.body(timetable)
}

#[get("/k8/{k8}/timetable.ical")]
pub async fn timetable_k8(
	k8: web::Path<String>,
	signer: web::Data<Signer>,
	clients: web::Data<Mutex<Clients>>,
) -> impl Responder {
	let timetable = handle_timetable_err_async(async move {
		let k8 = k8.into_inner();
		let credentials = crate::k8::decode_k8(&k8, &signer)
			.with_context(|| format!("failed to decode k8 {k8}"))?;

		let timetable = super::generic_timetable(&credentials, clients).await?;
		anyhow::Ok(timetable)
	})
	.await;

	HttpResponse::Ok()
		.content_type("text/calendar")
		.body(timetable)
}
