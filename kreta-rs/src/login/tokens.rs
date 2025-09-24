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
