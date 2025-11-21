use actix_web::{HttpResponse, Responder, post, web};
use anyhow::{Context, anyhow};
use base64::Engine;
use credsign::Signer;
use kreta_rs::login::Credentials;

pub fn decode_k8(k8: &str, signer: &Signer) -> anyhow::Result<Credentials> {
	let credentials = signer
		.decrypt_text(&k8)
		.with_context(|| "while decrypting k8")?;
	let credentials = {
		let mut credentials = credentials.split('\n').map(String::from);
		let username = credentials
			.next()
			.ok_or_else(|| anyhow!("invalid syntax for k8: first line is username"))?;
		let passwd = credentials
			.next()
			.ok_or_else(|| anyhow!("invalid syntax for k8: second line is password"))?;
		let inst_id = credentials
			.next()
			.ok_or_else(|| anyhow!("invalid syntax for k8: third line is institute id"))?;

		(username, passwd, inst_id)
	};
	let (username, passwd, inst_id) = credentials;

	let credentials = Credentials::new(inst_id, username, passwd);
	Ok(credentials)
}

pub fn decode_base64(base64: &str) -> anyhow::Result<Credentials> {
	let blob = base64::prelude::BASE64_URL_SAFE
		.decode(base64)
		.with_context(|| "while decoding the base64 blob provided")?;
	let blob = String::from_utf8(blob).with_context(|| "base64 encoded blob is not utf-8")?;

	let mut credentials = blob.split('\n').map(String::from);
	let username = credentials
		.next()
		.ok_or_else(|| anyhow!("invalid syntax for the base64 blob: first line is username"))?;
	let passwd = credentials
		.next()
		.ok_or_else(|| anyhow!("invalid syntax for the base64 blob: second line is password"))?;
	let inst_id = credentials
		.next()
		.ok_or_else(|| anyhow!("invalid syntax for the base64 blob: third line is institute id"))?;

	Ok(Credentials::new(inst_id, username, passwd))
}

#[post("/create_k8")]
pub async fn create_k8(signer: web::Data<Signer>, credentials: String) -> impl Responder {
	let encrypted = match signer.encrypt_text(&credentials) {
		Ok(a) => a,
		Err(err) => return HttpResponse::NotAcceptable().body(format!("{err:#?}")),
	};

	HttpResponse::Ok().body(encrypted)
}
