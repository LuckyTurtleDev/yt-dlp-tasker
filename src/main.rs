use std::fs;

use serde::Deserialize;
mod serde_helper;
use serde_helper::*;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Download {
	///url to dowload videos.
	/// Urls of inner vec treatet as url in the same group. outer vec share only the same config
	url: Vec<VecOrOne<String>>
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Config {
	/// path os yt-dlp binary (default: `yt-dlp`)
	#[serde(default = "default_bin_name")]
	bin_name: String,
	#[serde(default)]
	audio_args: Vec<String>,
	#[serde(default)]
	video_args: Vec<String>,
	download: Vec<Download>
}

fn default_bin_name() -> String {
	"yt-dlp".into()
}

fn main() -> anyhow::Result<()> {
	let config: Config = basic_toml::from_str(&fs::read_to_string("config.toml")?)?;
	println!("{config:#?}");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn config() {
		let _: Config = basic_toml::from_str(include_str!("../config.toml")).unwrap();
	}
}
