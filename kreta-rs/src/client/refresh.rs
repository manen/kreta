use std::borrow::Cow;

use serde::Serialize;

#[cfg(feature = "client")]
use crate::login::TokensRaw;

#[derive(Clone, Debug, Serialize)]
pub struct RefreshTokenBody<'a> {
	institute_code: Cow<'a, str>,
	refresh_token: Cow<'a, str>,
	grant_type: Cow<'a, str>,
	client_id: Cow<'a, str>,
}

#[cfg(feature = "client")]
impl crate::client::Client {
	/// uses the refresh token to get a new access token but doesn't rewrite itself to use the new access
	/// token so this probably isn't something you should use
	async fn refresh_new(&self) -> anyhow::Result<TokensRaw> {
		use anyhow::Context;

		use crate::login::login_flow::CLIENT_ID;

		let grant_type = "refresh_token";
		let body = RefreshTokenBody {
			institute_code: (&self.inst_id).into(),
			refresh_token: (&self.tokens.refresh_token).into(),
			grant_type: grant_type.into(),
			client_id: CLIENT_ID.into(),
		};

		let req = self
			.client
			.post("https://idp.e-kreta.hu/connect/token")
			.header(
				"User-Agent",
				"hu.ekreta.student/5.8.0+2025082301/SM-S9280/9/28",
			)
			.form(&body)
			.build()?;
		let resp = self.client.execute(req).await?;

		let status_code = resp.status();
		if !status_code.is_success() {
			use anyhow::anyhow;

			let body = resp.text().await?;
			let err = anyhow!(
				"using the refresh token to get a new access token, received non-ok status code: {status_code}\n{body}"
			);
			return Err(err);
		}

		let tokens: TokensRaw = resp.json().await.with_context(|| {
			format!(
				"everything went fine but the body of the /connect/token endpoint isn't valid json"
			)
		})?;
		Ok(tokens)
	}

	pub async fn refresh(&mut self) -> anyhow::Result<()> {
		use anyhow::Context;
		use std::time::{Duration, Instant};

		let tokens = self
			.refresh_new()
			.await
			.with_context(|| "while getting new access token (Client.refresh())")?;

		self.access_expires = Instant::now() + Duration::from_secs(tokens.expires_in.abs() as _);
		self.tokens = tokens;

		Ok(())
	}

	pub async fn refresh_if_needed(&mut self) -> anyhow::Result<()> {
		use std::time::Instant;

		if self.access_expires <= Instant::now() {
			self.refresh().await?;
		}
		Ok(())
	}
}
