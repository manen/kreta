use anyhow::{Context, anyhow};

use crate::login_flow::LoginFlow;

pub mod credentials;
pub mod login_flow;

fn main() {
	println!("Hello, world!");

	let rt = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap();

	rt.block_on(start()).unwrap()
}

async fn start() -> anyhow::Result<()> {
	let credentials = tokio::fs::read_to_string("./credentials.txt")
		.await
		.with_context(|| format!("make sure to place your credentials into ./credentials.txt"))?;

	let mut credentials = credentials.split('\n');

	let username = credentials
		.next()
		.ok_or_else(|| anyhow!("first line of credentials.txt should be username"))?;
	let passwd = credentials
		.next()
		.ok_or_else(|| anyhow!("second line of credentials.txt should be password"))?;
	let inst_id = credentials
		.next()
		.ok_or_else(|| anyhow!("third line of credentials.txt should be institute id"))?;

	let login_flow = LoginFlow::new()?;

	let data = login_flow.begin().await?;
	println!("{data:#?}");

	Ok(())
}
