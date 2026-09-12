use std::collections::HashMap;

use anyhow::Context;
use kreta_rs::client::Client;

use crate::{
	Preprocessed,
	process::{process_absences, process_exams, process_homework, process_timetable},
};

pub async fn get_preprocessed_range(
	client: &Client,
	from: chrono::DateTime<chrono::Utc>,
	to: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<Preprocessed> {
	use futures::StreamExt;
	let timetable = async {
		let mut buf = Vec::new();
		let mut stream = client.timetable_range_stream(from.clone(), to.clone());
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading chunk from timetable_range_stream")?;
			process_timetable(&mut buf, next);
		}

		anyhow::Ok(buf)
	};
	let homework = async {
		let mut buf = HashMap::new();
		let mut stream = client.homework_range_stream(from.clone(), to.clone());
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading chunk from homework_range_stream")?;
			process_homework(&mut buf, next)
				.with_context(|| "while processing chunk from homework_range_stream")?;
		}

		anyhow::Ok(buf)
	};
	let exams = async {
		let mut buf = HashMap::new();
		let mut stream = client.exams_range_stream(from.clone(), to.clone());
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading chunk from exams_range_stream")?;
			process_exams(&mut buf, next);
		}

		anyhow::Ok(buf)
	};
	let absences = async {
		let mut buf = HashMap::new();
		let mut stream = client.absences_range_stream(from.clone(), to.clone());
		while let Some(next) = stream.next().await {
			let next = next.with_context(|| "while reading chunk from absences_range_stream")?;
			process_absences(&mut buf, next)
				.with_context(|| "while processing chunk from absences_range_stream")?;
		}
		anyhow::Ok(buf)
	};

	let (timetable, homework, exams, absences) = tokio::join!(timetable, homework, exams, absences);
	let (timetable, homework, exams, absences) = (
		timetable.with_context(|| format!("while querying lessons between {from:?} and {to:?}"))?,
		homework.with_context(|| format!("while querying homework between {from:?} and {to:?}"))?,
		exams.with_context(|| format!("while querying exams between {from:?} and {to:?}"))?,
		absences.with_context(|| format!("while querying absences between {from:?} and {to:?}"))?,
	);

	anyhow::Ok((timetable, homework, exams, absences))
}
