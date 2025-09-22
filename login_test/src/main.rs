use kreta_rs::{client::Client, login::LoginFlow};

mod creds_from_file;

fn main() {
	println!("Hello, world!");

	let rt = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap();

	rt.block_on(start()).unwrap()
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

async fn start() -> anyhow::Result<()> {
	let credentials = creds_from_file::read_from_file("./credentials.txt").await?;
	let client = Client::full_login(&credentials).await?;

	Ok(())
}
