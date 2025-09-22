use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Tokens {
	id_token: String,
	access_token: String,
	expires_in: i32,
	token_type: String,
	refresh_token: String,
	scope: String,
}
