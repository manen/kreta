use std::{borrow::Cow, collections::HashMap};

use anyhow::{Context, anyhow};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::Digest;

pub const CLIENT_ID: &str = "kreta-ellenorzo-student-mobile-android";

use super::credentials::Credentials;

/// a login flow implementation that does not require the user to open kreta's website to log in;
/// we kinda cheat our way around it by parsing the login page and filing the post request manually, as if it was sent from the browser
///
/// it's unknown how stable this method will be in the future but for now it more than does the job \
/// (i'll probably look into doing oauth the correct way but it's still a question whether kreta will accept any arbitrary redirect_uri or only the ones it likes)
pub struct LoginFlow {
	client: reqwest::Client,
}
impl LoginFlow {
	pub fn new() -> anyhow::Result<Self> {
		let client = reqwest::Client::builder()
			.redirect(reqwest::redirect::Policy::limited(5))
			.https_only(true)
			.cookie_store(true)
			.user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36")
			.build()?;

		Ok(Self { client })
	}
}

// --- oauth implementation start

#[derive(Clone, Debug)]
/// all the data we have when we successfully requested and parsed the authentication page
pub struct BeginData {
	verifier: String,
	return_url: String,
	verification_token: String,
}

fn random_base64() -> String {
	type T = [u8; 16];

	let val: T = rand::random();

	let token = BASE64_URL_SAFE_NO_PAD.encode(&val);
	token
}

/// -> (verifier, challenge)
fn challenge() -> (String, String) {
	type Verifier = [u8; 64];

	let verifier_bytes: Verifier = rand::random();
	let verifier = BASE64_URL_SAFE_NO_PAD.encode(&verifier_bytes);

	let digest = sha2::Sha256::digest(verifier.as_bytes());
	let challenge = BASE64_URL_SAFE_NO_PAD.encode(digest);

	(verifier, challenge)
}

impl LoginFlow {
	async fn begin_send(&self) -> anyhow::Result<(String, String)> {
		let state = random_base64();
		let nonce = random_base64();

		let (verifier, challenge) = challenge();

		let req = self.client.get(format!("https://idp.e-kreta.hu/connect/authorize?redirect_uri=https://mobil.e-kreta.hu/ellenorzo-student/prod/oauthredirect&client_id={CLIENT_ID}&response_type=code&prompt=login&state={state}&nonce={nonce}&scope=openid email offline_access kreta-ellenorzo-webapi.public kreta-eugyintezes-webapi.public kreta-fileservice-webapi.public kreta-mobile-global-webapi.public kreta-dkt-webapi.public kreta-ier-webapi.public&code_challenge={challenge}&code_challenge_method=S256")).build()?;
		let resp = self.client.execute(req).await?;
		let resp = resp.error_for_status()?;

		let body = resp.text().await?;

		Ok((verifier, body))
	}

	/// requests the login screen and parses it, extracting whatever it is we need
	pub async fn begin(&self) -> anyhow::Result<BeginData> {
		let (verifier, body) = self
			.begin_send()
			.await
			.with_context(|| format!("error while sending begin request"))?;

		let document = Html::parse_document(&body);

		let return_selector =
			Selector::parse("input[id=\"ReturnUrl\"]").map_err(|a| anyhow!("{a}"))?;
		let return_url = document
			.select(&return_selector)
			.next()
			.and_then(|a| a.value().attr("value"));
		let return_url = return_url.ok_or_else(|| {
			anyhow!("returned page doesn't have a ReturnUrl element ({return_selector:?})")
		})?;

		let token_selector = Selector::parse("input[name=\"__RequestVerificationToken\"]")
			.map_err(|a| anyhow!("{a}"))?;
		let verification_token = document
			.select(&token_selector)
			.next()
			.and_then(|a| a.value().attr("value"));
		let verification_token = verification_token.ok_or_else(|| {
			anyhow!(
				"returned page doesn't have a __RequestVerificationToken element ({token_selector:?})"
			)
		})?;

		let data = BeginData {
			verifier: verifier.into(),
			return_url: return_url.into(),
			verification_token: verification_token.into(),
		};

		Ok(data)
	}
}

#[derive(Clone, Debug, Serialize)]
/// the body of the post request we send to https://idp.e-kreta.hu/account/login
pub struct LoginBody<'a> {
	#[serde(rename = "ReturnUrl")]
	return_url: Cow<'a, str>,
	#[serde(rename = "__RequestVerificationToken")]
	request_verification_token: Cow<'a, str>,
	#[serde(rename = "UserName")]
	username: Cow<'a, str>,
	#[serde(rename = "Password")]
	passwd: Cow<'a, str>,
	#[serde(rename = "InstituteCode")]
	inst_code: Cow<'a, str>,

	#[serde(rename = "loginType")]
	/// just leave it as "InstituteLogin"
	login_type: Cow<'a, str>,
	#[serde(rename = "ClientId")]
	client_id: Cow<'a, str>,

	#[serde(rename = "IsTemporaryLogin")]
	/// either "True" or "False"
	is_temporary_login: Cow<'a, str>,
}
impl<'a> LoginBody<'a> {
	pub fn new_explicit(
		return_url: impl Into<Cow<'a, str>>,
		request_verification_token: impl Into<Cow<'a, str>>,
		username: impl Into<Cow<'a, str>>,
		passwd: impl Into<Cow<'a, str>>,
		inst_code: impl Into<Cow<'a, str>>,
		client_id: impl Into<Cow<'a, str>>,
		is_temporary_login: bool,
	) -> Self {
		let return_url = return_url.into();
		let request_verification_token = request_verification_token.into();
		let username = username.into();
		let passwd = passwd.into();
		let inst_code = inst_code.into();
		let client_id = client_id.into();

		let is_temporary_login = if is_temporary_login {
			"True".into()
		} else {
			"False".into()
		};
		let login_type = "InstituteLogin".into();

		Self {
			return_url,
			request_verification_token,
			username,
			passwd,
			inst_code,
			login_type,
			client_id,
			is_temporary_login,
		}
	}
	pub fn new(
		return_url: impl Into<Cow<'a, str>>,
		request_verification_token: impl Into<Cow<'a, str>>,
		credentials: &'a Credentials,
		client_id: impl Into<Cow<'a, str>>,
		is_temporary_login: bool,
	) -> Self {
		let username = credentials.username();
		let passwd = credentials.passwd();
		let inst_id = credentials.inst_id();

		Self::new_explicit(
			return_url,
			request_verification_token,
			username,
			passwd,
			inst_id,
			client_id,
			is_temporary_login,
		)
	}
}

impl LoginFlow {
	pub async fn post_credentials(
		&self,
		begin_data: &BeginData,
		credentials: &Credentials,
	) -> anyhow::Result<()> {
		let login_body = LoginBody::new(
			&begin_data.return_url,
			&begin_data.verification_token,
			credentials,
			"",
			false,
		);

		self.post_credentials_map(&login_body).await
	}
	/// meant to be used with [LoginBody] but u do whatever u want lowkey
	pub async fn post_credentials_map<M: Serialize>(&self, map: &M) -> anyhow::Result<()> {
		let req = self
			.client
			.post("https://idp.e-kreta.hu/account/login")
			// .post("https://adgadgadgadg.free.beeceptor.com/babab")
			.form(map)
			.build()?;
		let resp = self.client.execute(req).await?;

		let status_code = resp.status();
		if !status_code.is_success() {
			let body = resp.text().await?;
			let err =
				anyhow!("post_credentials received non-ok status code: {status_code}\n{body}");
			return Err(err);
		}

		Ok(())
	}
}

#[derive(Clone, Debug, Serialize)]
/// the data we send to https://idp.e-kreta.hu/connect/token
pub struct ConnectTokenBody<'a> {
	code: Cow<'a, str>,
	grant_type: Cow<'a, str>,
	redirect_uri: Cow<'a, str>,
	code_verifier: Cow<'a, str>,
	client_id: Cow<'a, str>,
}
impl<'a> ConnectTokenBody<'a> {
	pub fn new_explicit(
		code: impl Into<Cow<'a, str>>,
		grant_type: impl Into<Cow<'a, str>>,
		redirect_uri: impl Into<Cow<'a, str>>,
		code_verifier: impl Into<Cow<'a, str>>,
		client_id: impl Into<Cow<'a, str>>,
	) -> Self {
		let code = code.into();
		let grant_type = grant_type.into();
		let redirect_uri = redirect_uri.into();
		let code_verifier = code_verifier.into();
		let client_id = client_id.into();

		Self {
			code,
			grant_type,
			redirect_uri,
			code_verifier,
			client_id,
		}
	}
}

impl LoginFlow {
	/// returns the url the return url forwarded the client to (see /thanks.py if all this is confusing)
	async fn resolve_return_url(&self, begin_data: &BeginData) -> anyhow::Result<String> {
		let basic_return_url = format!("https://idp.e-kreta.hu{}", begin_data.return_url);

		let req = self.client.get(&basic_return_url).build()?;
		let resp = self.client.execute(req).await?;
		let resp = resp.error_for_status()?;

		Ok(resp.url().as_str().into())
	}
	async fn resolve_return_url_code(&self, begin_data: &BeginData) -> anyhow::Result<String> {
		let resolved_return_url = self.resolve_return_url(begin_data).await?;

		let code = resolved_return_url.split("code=").nth(1).ok_or_else(|| anyhow!("url returned by LoginFlow.resolve_return_url is invalid, as there's no code= segment\n{resolved_return_url}"))?;
		let code = code.split('&').next().ok_or_else(|| anyhow!("url returned by LoginFlow.resolve_return_url is invalid, as there's no & symbol after code=\n{resolved_return_url}"))?;

		Ok(code.into())
	}

	pub async fn request_token(&self, begin_data: &BeginData) -> anyhow::Result<Tokens> {
		let code = self.resolve_return_url_code(begin_data).await?;
		let grant_type = "authorization_code";
		let redirect_uri = "https://mobil.e-kreta.hu/ellenorzo-student/prod/oauthredirect";
		let code_verifier = &begin_data.verifier;

		let connect_token_body = ConnectTokenBody::new_explicit(
			code,
			grant_type,
			redirect_uri,
			code_verifier,
			CLIENT_ID,
		);

		self.request_token_map(&connect_token_body).await
	}
	pub async fn request_token_map<S: Serialize>(&self, map: &S) -> anyhow::Result<Tokens> {
		let req = self
			.client
			.post("https://idp.e-kreta.hu/connect/token")
			.header(
				"User-Agent",
				"hu.ekreta.student/5.8.0+2025082301/SM-S9280/9/28",
			)
			.form(map)
			.build()?;

		let resp = self.client.execute(req).await?;
		let status_code = resp.status();
		if !status_code.is_success() {
			let body = resp.text().await?;
			let err =
				anyhow!("post_credentials received non-ok status code: {status_code}\n{body}");
			return Err(err);
		}

		let tokens: Tokens = resp.json().await.with_context(|| {
			format!(
				"everything went fine but the body of the /connect/token endpoint isn't valid json"
			)
		})?;

		Ok(tokens)
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct Tokens {
	id_token: String,
	access_token: String,
	expires_in: i32,
	token_type: String,
	refresh_token: String,
	scope: String,
}
