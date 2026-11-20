use anyhow::{Context, anyhow};

#[cfg(feature = "timerange")]
#[cfg(feature = "client")]
use crate::types::HomeworkRaw;

#[cfg(feature = "client")]
use super::Client;

#[cfg(feature = "client")]
impl Client {
	/// https://nzx.hu/kreta-api/mobileapi/gethomeworks \
	/// from & to are both yyyy-mm-dd
	/// maximum distance between from & to is 3 weeks
	pub async fn homework(&self, from: &str, to: &str) -> anyhow::Result<Vec<HomeworkRaw>> {
		let url = format!(
			"https://{}.e-kreta.hu/ellenorzo/v3/Sajat/HaziFeladatok?datumTol={from}&datumIg={to}",
			self.inst_id
		);
		let req = self
			.client
			.get(&url)
			.bearer_auth(self.access_token())
			.build()?;

		let resp = self.client.execute(req).await?;
		let status_code = resp.status();
		if !status_code.is_success() {
			let body = resp.text().await?;
			let err = anyhow!("{url} returned {status_code}\n{body}");
			return Err(err);
		}

		let resp = resp.text().await?;
		// println!("{resp}");
		let resp: Vec<HomeworkRaw> = serde_json::from_str(&resp)
			.with_context(|| format!("failed to deserialize response from {url}"))?;

		Ok(resp)
	}
}

#[cfg(feature = "client")]
#[cfg(feature = "timerange")]
impl Client {
	/// sets up a new FuturesUnordered with all of the chunks inside but doesn't start polling yet
	/// so no need for this function to be async
	pub fn homework_range_stream(
		&self,
		from: chrono::DateTime<chrono::Utc>,
		to: chrono::DateTime<chrono::Utc>,
	) -> impl futures::Stream<Item = anyhow::Result<Vec<HomeworkRaw>>> {
		use futures::stream::FuturesUnordered;

		let timesplit = timerange::range(
			from,
			to,
			chrono::Duration::weeks(3),
			chrono::Duration::days(1),
		);

		let mut stream = FuturesUnordered::new();
		stream.extend(timesplit.map(|(from, to)| async move {
			let from = from.format("%Y-%m-%d").to_string();
			let to = to.format("%Y-%m-%d").to_string();

			let homework = self.homework(&from, &to).await?;
			anyhow::Ok(homework)
		}));

		stream
	}

	/// homework query with no maximum distance between from & to
	pub async fn homework_range(
		&self,
		from: chrono::DateTime<chrono::Utc>,
		to: chrono::DateTime<chrono::Utc>,
	) -> anyhow::Result<Vec<HomeworkRaw>> {
		use futures::StreamExt;

		let mut buf = Vec::new();

		let mut stream = self.homework_range_stream(from, to);
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading homework from timerange stream")?;
			buf.extend(next);
		}

		Ok(buf)
	}
}
