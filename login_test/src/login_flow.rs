use std::{borrow::Cow, collections::HashMap};

use anyhow::{Context, anyhow};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use reqwest::{
	StatusCode,
	header::{HeaderMap, HeaderName, HeaderValue},
};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::credentials::Credentials;

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

	async fn begin_send(&self) -> anyhow::Result<(String, String)> {
		let state = random_base64();
		let nonce = random_base64();

		let (verifier, challenge) = challenge();

		let req = self.client.get(format!("https://idp.e-kreta.hu/connect/authorize?redirect_uri=https://mobil.e-kreta.hu/ellenorzo-student/prod/oauthredirect&client_id=kreta-ellenorzo-student-mobile-android&response_type=code&prompt=login&state={state}&nonce={nonce}&scope=openid email offline_access kreta-ellenorzo-webapi.public kreta-eugyintezes-webapi.public kreta-fileservice-webapi.public kreta-mobile-global-webapi.public kreta-dkt-webapi.public kreta-ier-webapi.public&code_challenge={challenge}&code_challenge_method=S256")).build()?;
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
