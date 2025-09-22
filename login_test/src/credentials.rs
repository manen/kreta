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
