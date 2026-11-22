use std::io::Cursor;

use anyhow::Context;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce, aead::Aead};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::DatabaseEntryCompatible;

#[derive(Clone, Debug, Hash, Serialize, Deserialize, Default)]
pub enum Encrypted<T: DatabaseEntryCompatible> {
	#[default]
	Uninitialized,
	HasData {
		nonce: Vec<u8>,
		encrypted_cbor_data: Vec<u8>,

		_t: std::marker::PhantomData<T>,
	},
}
impl<T: DatabaseEntryCompatible> Encrypted<T> {
	pub fn new<K: Serialize>(data: T, key: &K) -> anyhow::Result<Self> {
		let mut cbor_buf = Vec::new();
		ciborium::into_writer(&data, &mut cbor_buf).with_context(|| {
			format!(
				"failed to serialize data inside of an Encrypted<{}>",
				std::any::type_name::<T>()
			)
		})?;

		let (nonce, encrypted) = encrypt(key, &cbor_buf).with_context(|| {
			format!(
				"failed to encrypt data inside of an Encrypted<{}>",
				std::any::type_name::<T>(),
			)
		})?;

		Ok(Self::HasData {
			nonce,
			encrypted_cbor_data: encrypted,
			_t: Default::default(),
		})
	}

	pub fn take<K: Serialize>(self, key: &K) -> anyhow::Result<T> {
		match self {
			Self::Uninitialized => Ok(T::default()),
			Self::HasData {
				nonce,
				encrypted_cbor_data,
				..
			} => {
				let cbor_data = decrypt(key, &nonce, &encrypted_cbor_data).with_context(|| {
					format!(
						"failed to decrypt data inside of an Encrypted<{}>",
						std::any::type_name::<T>()
					)
				})?;

				let data: T = ciborium::from_reader(Cursor::new(cbor_data)).with_context(|| {
					format!(
						"failed to deserialize decrypted data inside of an Encrypted<{}>",
						std::any::type_name::<T>()
					)
				})?;

				Ok(data)
			}
		}
	}
}

impl<T: DatabaseEntryCompatible> Encrypted<T> {
	pub fn map<K: Serialize>(self, key: &K, f: impl FnOnce(T) -> T) -> anyhow::Result<Self> {
		let data = self.take(key)?;
		let new = f(data);
		Self::new(new, key)
	}
	pub async fn map_async<K: Serialize>(
		self,
		key: &K,
		f: impl AsyncFnOnce(T) -> T,
	) -> anyhow::Result<Self> {
		let data = self.take(key)?;
		let new = f(data).await;
		Self::new(new, key)
	}

	pub async fn with_mut_async<K: Serialize, R>(
		&mut self,
		key: &K,
		f: impl AsyncFnOnce(&mut T) -> R,
	) -> anyhow::Result<R> {
		let self_taken = std::mem::replace(self, Default::default());

		let mut to_return = None;
		let new = self_taken
			.map_async(key, async |mut data| {
				to_return = Some(f(&mut data).await);
				data
			})
			.await?;

		*self = new;
		let to_return = to_return
			.expect("this function is broken as to_return should never be None (f wasn't ran)");
		Ok(to_return)
	}
}

// Derive a 32-byte encryption key from ANY serializable datatype.
fn derive_key<T: Serialize>(key: &T) -> [u8; 32] {
	Sha256::digest(serde_json::to_vec(key).unwrap()).into()
}

/// returns nonce, cipher
fn encrypt<T: Serialize>(key: &T, plaintext: &[u8]) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
	let cipher = XChaCha20Poly1305::new(&derive_key(key).into());

	let mut nonce = [0u8; 24];
	rand::fill(&mut nonce);

	let ciphertext = cipher.encrypt(&XNonce::try_from(nonce)?, plaintext)?;

	Ok((nonce.to_vec(), ciphertext))
}

fn decrypt<T: Serialize>(key: &T, nonce: &[u8], ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
	let cipher = XChaCha20Poly1305::new(&derive_key(key).into());

	let plaintext = cipher.decrypt(&XNonce::try_from(nonce)?, ciphertext)?;

	Ok(plaintext)
}

#[allow(unused)]
fn assert_compatability() {
	fn a<T: DatabaseEntryCompatible>() {}
	a::<Encrypted<String>>();
}
