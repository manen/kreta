#[derive(Clone, Debug)]
pub struct Credentials {
	inst_id: String,
	username: String,
	passwd: String,
}
impl Credentials {
	pub fn new(inst_id: String, username: String, passwd: String) -> Self {
		Self {
			inst_id,
			username,
			passwd,
		}
	}

	pub fn inst_id(&self) -> &str {
		&self.inst_id
	}
	pub fn username(&self) -> &str {
		&self.username
	}
	pub fn passwd(&self) -> &str {
		&self.passwd
	}
}

// --

use anyhow::{Context, anyhow};
use std::path::Path;

impl Credentials {
	/// syntax of the file being read (lines split only by newlines):
	/// username
	/// password
	/// institute code
	pub async fn read_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
		let credentials = tokio::fs::read_to_string(path).await.with_context(|| {
			format!("make sure to place your credentials into ./credentials.txt")
		})?;

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

		Ok(Self::new(inst_id, username, passwd))
	}
}
