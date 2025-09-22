use std::path::Path;

use anyhow::{Context, anyhow};
use kreta_rs::login::Credentials;

/// syntax of the file being read (lines split only by newlines):
/// username
/// password
/// institute code
pub async fn read_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Credentials> {
	let credentials = tokio::fs::read_to_string(path)
		.await
		.with_context(|| format!("make sure to place your credentials into ./credentials.txt"))?;

	let mut credentials = credentials.split('\n').map(String::from);

	let username = credentials
		.next()
		.ok_or_else(|| anyhow!("first line of credentials.txt should be username"))?;
	let passwd = credentials
		.next()
		.ok_or_else(|| anyhow!("second line of credentials.txt should be password"))?;
	let inst_id = credentials
		.next()
		.ok_or_else(|| anyhow!("third line of credentials.txt should be institute id"))?;

	Ok(Credentials::new(inst_id, username, passwd))
}
