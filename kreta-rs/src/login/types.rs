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

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct TokensRaw {
	pub(crate) id_token: String,
	pub(crate) access_token: String,
	pub(crate) expires_in: i32,
	pub(crate) token_type: String,
	pub(crate) refresh_token: String,
	pub(crate) scope: String,
}
