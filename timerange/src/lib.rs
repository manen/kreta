use chrono::NaiveDate;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// iterator that splits up time into pieces with a maximum size and optional distance in between
pub struct Timesplit {
	i: NaiveDate,
	end: NaiveDate,

	max_dur: chrono::Duration,
	/// the amount of time between a `to` output and the next `from` output
	gap: chrono::Duration,
}
impl Iterator for Timesplit {
	type Item = (NaiveDate, NaiveDate);

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.end.clone() - self.i.clone();
		let res = remaining.as_seconds_f32() / self.max_dur.as_seconds_f32();
		let res = res.min(0.0).ceil() as usize;

		(res, Some(res))
	}
	fn next(&mut self) -> Option<Self::Item> {
		let from = self.i.clone();
		let to = (from.clone() + self.max_dur).min(self.end.clone());

		if from >= to {
			return None;
		}
		self.i = to.clone() + self.gap;

		Some((from, to))
	}
}

/// creates a new range of times \
/// usage from here is similar to std::ops::Range, just an iterator
pub fn range(
	from: NaiveDate,
	to: NaiveDate,
	max_duration: chrono::Duration,
	gap: chrono::Duration,
) -> Timesplit {
	let dur = if max_duration < chrono::Duration::days(1) {
		chrono::Duration::days(1)
	} else {
		max_duration
	};

	Timesplit {
		i: from,
		end: to,
		max_dur: dur,
		gap,
	}
}

// i'll get back to it whenever
// combine is proven to work tho which is cool
