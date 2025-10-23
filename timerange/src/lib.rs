use chrono::TimeZone;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Timesplit<Tz: TimeZone> {
	i: chrono::DateTime<Tz>,
	end: chrono::DateTime<Tz>,

	max_dur: chrono::Duration,
}
impl<Tz: TimeZone> Iterator for Timesplit<Tz> {
	type Item = (chrono::DateTime<Tz>, chrono::DateTime<Tz>);

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.end.clone() - self.i.clone();
		let res = remaining.as_seconds_f32() / self.max_dur.as_seconds_f32();
		let res = res.min(0.0) as usize;

		(res, Some(res))
	}
	fn next(&mut self) -> Option<Self::Item> {
		let from = self.i.clone();
		let to = (from.clone() + self.max_dur).min(self.end.clone());

		if from >= to {
			return None;
		}
		self.i = to.clone();

		Some((from, to))
	}
}

// i'll get back to it whenever
// combine is proven to work tho which is cool

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use chrono::{DateTime, Utc};

	use super::*;

	#[test]
	fn test_name() {
		let i = dateparser::parse_with_timezone("2025-10-01 00:00:00", &Utc).unwrap();
		// println!("start: {i:?}");
		let end = dateparser::parse_with_timezone("2025-10-08 00:01:00", &Utc).unwrap();
		let max_dur = chrono::Duration::hours(12);

		let timesplit = Timesplit { i, end, max_dur };

		let timesplit = timesplit.collect::<Vec<_>>();

		let expected = vec![
			(
				DateTime::<Utc>::from_str("2025-10-01T00:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-01T12:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-01T12:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-02T00:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-02T00:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-02T12:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-02T12:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-03T00:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-03T00:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-03T12:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-03T12:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-04T00:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-04T00:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-04T12:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-04T12:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-05T00:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-05T00:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-05T12:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-05T12:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-06T00:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-06T00:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-06T12:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-06T12:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-07T00:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-07T00:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-07T12:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-07T12:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-08T00:00:00Z").unwrap(),
			),
			(
				DateTime::<Utc>::from_str("2025-10-08T00:00:00Z").unwrap(),
				DateTime::<Utc>::from_str("2025-10-08T00:01:00Z").unwrap(),
			),
		];
		assert_eq!(timesplit, expected);
	}
}
