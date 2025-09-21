use anyhow::{Context, anyhow};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use scraper::{Html, Selector};
use sha2::Digest;

pub struct LoginFlow {
	client: reqwest::Client,
}
impl LoginFlow {
	pub fn new() -> anyhow::Result<Self> {
		let client = reqwest::Client::builder()
			.redirect(reqwest::redirect::Policy::limited(1))
			.https_only(true)
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
