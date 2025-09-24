use std::{
	collections::HashMap,
	ops::{Deref, DerefMut},
	sync::Arc,
};

use kreta_rs::{
	client::Client,
	login::{Credentials, credentials},
};
use tokio::sync::{Mutex, MutexGuard};

#[derive(Default)]
/// essentially a client cache
pub struct Clients {
	map: HashMap<String, Arc<Mutex<Client>>>,
}
impl Clients {
	pub fn insert_client(
		&mut self,
		username: String,
		client: Client,
	) -> Option<Arc<Mutex<Client>>> {
		self.map.insert(username, Arc::new(Mutex::new(client)))
	}

	pub fn get_client(&self, username: &str) -> Option<Arc<Mutex<Client>>> {
		self.map.get(username).cloned()
	}

	/// either uses the saved client from the map, or logs in using the credentials
	pub async fn client(
		&mut self,
		credentials: &Credentials,
	) -> anyhow::Result<Arc<Mutex<Client>>> {
		let saved = self.get_client(credentials.username());
		if let Some(saved) = saved {
			let mut client = saved.lock().await;

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
		self.map
			.insert(credentials.username().into(), client.clone());
		Ok(client)
	}
}
