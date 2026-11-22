#[cfg(feature = "timerange")]
#[cfg(feature = "client")]
use chrono::NaiveDate;

#[cfg(feature = "client")]
use crate::{client::Client, types::AbsenceRaw};

#[cfg(feature = "client")]
impl Client {
	/// https://nzx.hu/kreta-api/mobileapi/getomissions \
	/// from & to are both yyyy-mm-dd
	/// maximum distance is 3 weeks
	pub async fn absences(&self, from: &str, to: &str) -> anyhow::Result<Vec<AbsenceRaw>> {
		// println!("fetching absences between {from} and {to}");

		use anyhow::Context;

		let url = format!(
			"https://{}.e-kreta.hu/ellenorzo/v3/sajat/Mulasztasok?datumTol={from}&datumIg={to}",
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
			use anyhow::anyhow;

			let body = resp.text().await?;
			let err = anyhow!("{url} returned {status_code}\n{body}");
			return Err(err);
		}

		let resp = resp.text().await?;
		let resp: Vec<AbsenceRaw> = serde_json::from_str(&resp)
			.with_context(|| format!("failed to deserialize response from {url}"))?;

		Ok(resp)
	}
}

#[cfg(feature = "client")]
#[cfg(feature = "timerange")]
impl Client {
	/// creates a pollable stream of the absences in chunks
	/// needs to be polled to start doing anything
	pub fn absences_range_stream(
		&self,
		from: NaiveDate,
		to: NaiveDate,
	) -> impl futures::Stream<Item = anyhow::Result<Vec<AbsenceRaw>>> {
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

			let absences = self.absences(&from, &to).await?;
			anyhow::Ok(absences)
		}));

		stream
	}

	/// absence query with no maximum distance between from & to
	pub async fn absences_range(
		&self,
		from: NaiveDate,
		to: NaiveDate,
	) -> anyhow::Result<Vec<AbsenceRaw>> {
		use futures::StreamExt as _;

		let mut buf = Vec::new();

		let mut stream = self.absences_range_stream(from, to);
		while let Some(next) = stream.next().await {
			use anyhow::Context;

			let next =
				next.with_context(|| format!("while reading absences from timerange stream"))?;
			buf.extend(next);
		}
		Ok(buf)
	}
}
