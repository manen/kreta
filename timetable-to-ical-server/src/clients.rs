use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use kreta_rs::{client::Client, login::Credentials};
use sha2::Digest;
use tokio::sync::Mutex;

#[derive(Default)]
/// essentially a client cache
pub struct Clients {
	/// k: username, v: (password hash, client)
	map: HashMap<String, (Vec<u8>, Arc<Mutex<Client>>)>,
}
impl Clients {
	/// either uses the saved client from the map, or logs in using the credentials
	pub async fn client(
		&mut self,
		credentials: &Credentials,
	) -> anyhow::Result<Arc<Mutex<Client>>> {
		// println!("{:#?}", self.map.len());

		// println!("retrieving client for {}", credentials.username());
		let saved = self.map.get(credentials.username()).cloned();
		// println!("saved: {}", saved.is_some());
		if let Some((passwd_hash, saved)) = saved {
			let mut client = saved.lock().await;

			// check incoming credentials with the ones we have saved, refuse without explanation if they're incorrect
			let incoming_passwd_hash = hash_password(credentials.passwd());
			if client.inst_id() != credentials.inst_id() || passwd_hash != incoming_passwd_hash {
				return Err(anyhow!("invalid credentials"));
			}

			let refresh_res = client.refresh_if_needed().await;
			// if refresh token fails just log in again
			match refresh_res {
				Ok(_) => {
					return Ok(saved.clone());
				}
				Err(err) => {
					eprintln!("failed to use saved client: {err}\ndefaulting to logging in again");
				}
			}
		}

		let client = Client::full_login(credentials).await?;
		let client = Arc::new(Mutex::new(client));
		let passwd_hash = hash_password(credentials.passwd());

		self.map
			.insert(credentials.username().into(), (passwd_hash, client.clone()));
		// println!("just saved client for {}", credentials.username());
		Ok(client)
	}
}

/// this would be an awful password hash function to use for any proper authentication service.
/// the reason i think it's probably fine is because it only stays in memory and only ever checked against the incoming password
/// of clients that have already previously authenticated successfully \
/// but still i'm happy to accept pull requests or whatever if memory security of self-hosted, (most likely) single-user apps is your thing
fn hash_password(passwd: &str) -> Vec<u8> {
	let mut hasher = sha2::Sha256::default();
	for _ in 0..42 {
		hasher.update(passwd);
	}
	let res = hasher.finalize();
	res.to_vec()
}
