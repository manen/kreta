use anyhow::{Context, anyhow};
#[cfg(feature = "timerange")]
#[cfg(feature = "client")]
use chrono::NaiveDate;

#[cfg(feature = "timerange")]
#[cfg(feature = "client")]
use crate::types::LessonRaw;

#[cfg(feature = "client")]
use super::Client;

#[cfg(feature = "client")]
impl Client {
	/// https://nzx.hu/kreta-api/mobileapi/getlessons \
	/// from & to are both expected to be in the format of yyyy-mm-dd \
	/// maximum distance between from & to is one month
	pub async fn timetable(&self, from: &str, to: &str) -> anyhow::Result<Vec<LessonRaw>> {
		use crate::types::LessonRaw;

		let url = format!(
			"https://{}.e-kreta.hu/ellenorzo/v3/sajat/OrarendElemek?datumTol={from}&datumIg={to}",
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
		let resp: Vec<LessonRaw> = serde_json::from_str(&resp).map_err(|err| {
			anyhow!(
				"{err}\n\nwhile deserializing response from {url}\n\nserver responded with:\n{resp}"
			)
		})?;

		Ok(resp)
	}
}

#[cfg(feature = "client")]
#[cfg(feature = "timerange")]
impl Client {
	/// sets up a new FuturesUnordered with all of the chunks inside but doesn't start polling yet
	/// so no need for this function to be async
	pub fn timetable_range_stream(
		&self,
		from: NaiveDate,
		to: NaiveDate,
	) -> impl futures::Stream<Item = anyhow::Result<Vec<LessonRaw>>> {
		use futures::stream::FuturesUnordered;

		let timesplit = timerange::range(
			from,
			to,
			chrono::Duration::days(30),
			chrono::Duration::days(0),
		);

		let mut stream = FuturesUnordered::new();
		stream.extend(timesplit.map(|(from, to)| async move {
			let from = from.format("%Y-%m-%d").to_string();
			let to = to.format("%Y-%m-%d").to_string();

			let timetable = self.timetable(&from, &to).await?;
			anyhow::Ok(timetable)
		}));

		stream
	}

	/// timetable query with no maximum distance between from & to
	pub async fn timetable_range(
		&self,
		from: NaiveDate,
		to: NaiveDate,
	) -> anyhow::Result<Vec<LessonRaw>> {
		use futures::StreamExt;

		let mut buf = Vec::new();

		let mut stream = self.timetable_range_stream(from, to);
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading lessons from timerange stream")?;
			buf.extend(next);
		}

		Ok(buf)
	}
}
