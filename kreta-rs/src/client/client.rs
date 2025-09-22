use anyhow::Context;

use crate::login::{Credentials, LoginFlow, Tokens};

/// the main client interface with which you can interact with the kreta api. \
/// an instance of this type has already logged in, but does not guarantee that the access token hasn't expired
pub struct Client {
	pub(crate) client: reqwest::Client,
	pub(crate) tokens: Tokens,
}

impl Client {
	pub fn new(client: reqwest::Client, tokens: Tokens) -> Self {
		Self { client, tokens }
	}

	/// completes the entire login sequence using [LoginFlow]
	pub async fn full_login(credentials: &Credentials) -> anyhow::Result<Self> {
		async fn internal(credentials: &Credentials) -> anyhow::Result<Client> {
			let login_flow = LoginFlow::new()?;

			let data = login_flow.begin().await?;
			login_flow.post_credentials(&data, credentials).await?;
			let tokens = login_flow.request_token(&data).await?;

			let client = login_flow.take_client();
			let client = Client { client, tokens };

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
