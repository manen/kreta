use anyhow::Context;
use base64::Engine;
use serde::Deserialize;
use timetable_to_ical::Options;

#[derive(Clone, Debug, Deserialize)]
pub struct OptsParams {
	/// base64 url encoded
	opts: Option<String>,
}
impl OptsParams {
	pub fn extract_options(&self) -> anyhow::Result<Options> {
		match &self.opts {
			Some(base64) => {
				let blob = base64::prelude::BASE64_URL_SAFE
					.decode(base64)
					.with_context(|| "while decoding the base64 blob provided as ?opts=")?;
				let blob = String::from_utf8(blob)
					.with_context(|| "base64 encoded blob provided as ?opts= is not utf-8")?;

				let decoded: Options = serde_json::from_str(&blob)
					.with_context(|| "while deserializing json options provided as ?opts=")?;

				Ok(decoded)
			}
			None => Ok(Default::default()),
		}
	}
}
