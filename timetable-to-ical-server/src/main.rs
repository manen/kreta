use actix_web::{App, HttpServer, Responder, get, web};

#[get("/")]
async fn index() -> impl Responder {
	"szia"
}

#[get("/basic/{inst_id}/{username}/{passwd}/timetable.ical")]
async fn timetable(path: web::Path<(String, String, String)>) -> impl Responder {
	let (inst_id, username, passwd) = path.into_inner();

	format!("{inst_id} {username} {passwd}")
}

const BIND: (&str, u16) = ("0.0.0.0", 8080);
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let server = HttpServer::new(move || App::new().service(index).service(timetable))
		.bind(BIND)?
		.run();
	println!("listening on http://{}:{}", BIND.0, BIND.1);

	server.await?;
	Ok(())
}
