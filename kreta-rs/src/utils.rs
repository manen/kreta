use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Deserializer};

pub fn deserialize_local_datetime<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
	D: Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?;

	let naive =
		NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S").map_err(serde::de::Error::custom)?;

	Local
		.from_local_datetime(&naive)
		.single()
		.ok_or_else(|| serde::de::Error::custom("ambiguous or invalid local datetime"))
}
