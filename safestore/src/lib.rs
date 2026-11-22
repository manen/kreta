//! the point is a disk db where i don't even know what's stored
//! so the name of the entries is a hashed secret and the content is encrypted with that secret
//!
//! in the kreta project this secret is most likely the user's password (hashed together with username and whatnot) because we don't have access to it only when a request actually comes in with a k8
//!
//! the way it's actually gonna work is that a Db<String> is going to store the string unencrypted
//! but a Db<Encrypted<String>> is gonna encrypt it so it's done with a wrapper type for simplicity
//! (and ofc the String is any type that implements DatabaseEntryCompatible)

mod encrypt;
pub use encrypt::*;

use std::{
	io::{Cursor, ErrorKind, SeekFrom},
	path::{Path, PathBuf},
};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use anyhow::Context;
use base64::{Engine, prelude::BASE64_URL_SAFE};
use serde::{Serialize, de::DeserializeOwned};

pub trait DatabaseEntryCompatible: Serialize + DeserializeOwned + Default {}
impl<T: Serialize + DeserializeOwned + Default> DatabaseEntryCompatible for T {}

/// a branch in a db and child dbs are all of a certain type \
/// just points to a folder
pub struct DbBunch<T: DatabaseEntryCompatible> {
	/// path of dir
	path: PathBuf,
	_t: std::marker::PhantomData<T>,
}
impl<T: DatabaseEntryCompatible> DbBunch<T> {
	pub async fn new(dir_path: impl Into<PathBuf>) -> anyhow::Result<Self> {
		let path = dir_path.into();

		tokio::fs::create_dir_all(&path)
			.await
			.with_context(|| format!("failed to create_dir_all db dir {}", path.display()))?;

		// let path = path
		// 	.canonicalize()
		let path = tokio::fs::canonicalize(&path)
			.await
			.with_context(|| format!("failed to canonicalize dbbunch path {}", path.display()))?;

		tokio::fs::read_dir(&path)
			.await
			.with_context(|| format!("failed to read db dir {}", path.display()))?
			.next_entry()
			.await
			.with_context(|| format!("failed to get first element of dir {}", path.display()))?;

		Ok(Self {
			path,
			_t: Default::default(),
		})
	}

	/// entry_id isn't a name but any type of binary input
	pub async fn db(&self, entry_id: &[u8]) -> anyhow::Result<Db<T>> {
		let b64 = BASE64_URL_SAFE.encode(&entry_id);

		let file_path = self.path.join(format!("{b64}.ssdb"));
		// let file_path = file_path
		// 	.canonicalize()
		// 	.with_context(|| format!("failed to canonicalize {}", file_path.display()))?;

		Db::from_path(&file_path).await
	}
}

pub struct Db<T: DatabaseEntryCompatible> {
	file: tokio::fs::File,
	current: T,
}
impl<T: DatabaseEntryCompatible> Db<T> {
	pub(crate) async fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
		let path = path.as_ref();
		let file = tokio::fs::OpenOptions::new() // todo this crate should use tokio and async
			.read(true)
			.write(true)
			.open(path)
			.await;

		// println!("{}", path.display());

		let (file, content) = match file {
			Ok(mut file) => {
				// read file

				let bufreader = tokio::fs::read(path)
					.await
					.with_context(|| format!("failed to read db file at {}", path.display()))?;
				let content: T = ciborium::from_reader(Cursor::new(bufreader))
					.with_context(|| format!("failed to deserialize db at {}", path.display()))?;

				file.seek(SeekFrom::Start(0)).await.with_context(|| {
					format!(
						"failed to seek back to start of the file after reading file for db at {}",
						path.display()
					)
				})?;

				(file, content)
			}
			Err(err) => match err.kind() {
				ErrorKind::NotFound => {
					let file_created = tokio::fs::OpenOptions::new()
						.read(true)
						.write(true)
						.create_new(true)
						.open(path)
						.await
						.with_context(|| {
							format!(
								"the db we're trying to open doesn't exist and it couldn't be created (at {})",
								path.display()
							)
						})?;
					let default: T = Default::default();

					(file_created, default)
				}
				_ => {
					return Err(err).with_context(|| {
						format!(
							"failed to open db at {} but not because it doesn't exist",
							path.display()
						)
					});
				}
			},
		};

		Ok(Self {
			file,
			current: content,
		})
	}

	pub async fn save(&mut self) -> anyhow::Result<()> {
		self.file
			.seek(SeekFrom::Start(0))
			.await
			.with_context(|| format!("failed to seek db to start"))?;

		let mut bufwriter = Vec::new();
		ciborium::into_writer(&self.current, &mut bufwriter)
			.with_context(|| format!("failed to serialize db"))?;

		self.file
			.write_all(&bufwriter)
			.await
			.with_context(|| "failed to write to db file")?;

		Ok(())
	}
}

impl<T: DatabaseEntryCompatible> Db<T> {
	pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
		f(&self.current)
	}
	pub async fn with_mut<R>(&mut self, f: impl AsyncFnOnce(&mut T) -> R) -> anyhow::Result<R> {
		let f = f(&mut self.current).await;
		self.save().await?;
		Ok(f)
	}
}
