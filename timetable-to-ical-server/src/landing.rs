use actix_web::{HttpResponse, Responder, get, web};

const LANDING_STATIC: &str = include_str!("../static/landing.html");
const STYLES_STATIC: &str = include_str!("../static/styles.css");

#[get("/")]
async fn index() -> impl Responder {
	web::Html::new(LANDING_STATIC)
}

#[get("/styles.css")]
async fn styles() -> impl Responder {
	HttpResponse::Ok()
		.content_type("text/css")
		.body(STYLES_STATIC)
}
