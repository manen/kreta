use actix_web::{HttpResponse, Responder, post, web};
use credsign::Signer;

#[post("/create_k8")]
pub async fn create_k8(signer: web::Data<Signer>, credentials: String) -> impl Responder {
	let encrypted = match signer.encrypt_text(&credentials) {
		Ok(a) => a,
		Err(err) => return HttpResponse::NotAcceptable().body(format!("{err:#?}")),
	};

	HttpResponse::Ok().body(encrypted)
}
