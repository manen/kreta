use std::path::Path;

use age::{
	secrecy::ExposeSecret,
	x25519::{Identity, Recipient},
};
use anyhow::anyhow;
use base64::{Engine, prelude::BASE64_URL_SAFE};

#[cfg(test)]
mod tests;

pub async fn load_or_create<P: AsRef<Path>>(cwd: P) -> anyhow::Result<Signer> {
	let cwd = cwd.as_ref();

	let path = cwd.join("./.k8");
	let file = tokio::fs::read_to_string(&path).await;
	let file = match file {
		Ok(a) => Some(a),
		Err(err) => match err.kind() {
			std::io::ErrorKind::NotFound => None,
			_ => Err(err)?,
		},
	};

	let key = match file {
		Some(a) => a,
		None => {
			let key = Identity::generate();
			let key_str = key.to_string().expose_secret().into();
			tokio::fs::create_dir_all(cwd).await?;
			tokio::fs::write(&path, &key_str).await?;
			key_str
		}
	};
	let key = key.parse().map_err(|err| anyhow!("{err}"))?;
	let signer = Signer::new(key);
	Ok(signer)
}

#[derive(Clone)]
pub struct Signer {
	key: Identity,
	pubkey: Recipient,
}
impl Signer {
	pub fn generate() -> Self {
		let identity = Identity::generate();
		Self::new(identity)
	}
	pub fn new(identity: Identity) -> Self {
		let pubkey = identity.to_public();
		Self {
			key: identity,
			pubkey,
		}
	}

	pub fn encrypt_text(&self, data: &str) -> anyhow::Result<String> {
		let encrypted = age::encrypt(&self.pubkey, data.as_bytes())?;

		let encrypted_str = BASE64_URL_SAFE.encode(&encrypted);
		Ok(encrypted_str)
	}
	pub fn decrypt_text(&self, encrypted: &str) -> anyhow::Result<String> {
		let bin = BASE64_URL_SAFE.decode(encrypted)?;

		let decrypted = age::decrypt(&self.key, &bin)?;
		let decrypted_str = String::from_utf8_lossy(&decrypted);
		Ok(decrypted_str.into_owned())
	}
}
