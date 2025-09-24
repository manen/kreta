use std::time::{Duration, Instant};

use anyhow::Context;

use crate::login::{Credentials, LoginFlow, TokensRaw, credentials};

/// the main client interface with which you can interact with the kreta api. \
/// an instance of this type has already logged in, but does not guarantee that the access token hasn't expired
pub struct Client {
	pub(crate) client: reqwest::Client,

	pub(crate) inst_id: String,
	pub(crate) tokens: TokensRaw,

	pub(crate) access_expires: Instant,
}

impl Client {
	/// refresh_if_needed won't work as expected if you wait a lot between getting your tokens and calling Client::new \
	/// you should probably use [Client::full_login] anyways
	pub fn new(client: reqwest::Client, inst_id: String, tokens: TokensRaw) -> Self {
		let access_expires = Instant::now() + Duration::from_secs(tokens.expires_in.abs() as _);

		Self {
			client,
			inst_id,
			tokens,
			access_expires,
		}
	}

	/// completes the entire login sequence using [LoginFlow]
	pub async fn full_login(credentials: &Credentials) -> anyhow::Result<Self> {
		println!("performing full login for {}", credentials.username());
		async fn internal(credentials: &Credentials) -> anyhow::Result<Client> {
			let login_flow = LoginFlow::new()?;

			let data = login_flow.begin().await?;
			login_flow.post_credentials(&data, credentials).await?;

			let tokens = login_flow.request_token(&data).await?;
			let access_expires = Instant::now() + Duration::from_secs(tokens.expires_in.abs() as _);

			let client = login_flow.take_client();
			let client = Client {
				client,
				inst_id: credentials.inst_id().into(),
				tokens,
				access_expires,
			};

			Ok(client)
		}

		internal(&credentials).await.with_context(|| {
			format!(
				"while logging in with credentials for {}",
				credentials.username()
			)
		})
	}
}

impl Client {
	pub(crate) fn access_token(&self) -> &str {
		&self.tokens.access_token
	}
}
