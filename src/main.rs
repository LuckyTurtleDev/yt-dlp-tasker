use std::{
	collections::HashMap,
	fs::{self, create_dir, create_dir_all},
	process::Command,
	thread::sleep,
	time::{Duration, Instant}
};

use anyhow::{bail, Context};
use serde::Deserialize;
mod serde_helper;
use serde_helper::*;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Download {
	/// Name of download job.
	/// The name is also used for the archive
	name: String,
	/// url to dowload videos.
	/// Can be a single entry or a vec
	#[serde(deserialize_with = "vec_or_one")]
	profile: Vec<String>,
	/// Video url to be downloaded. You can use anything here which is supported by yt-dlp.
	/// Can be a single entry or a vec
	#[serde(deserialize_with = "vec_or_one")]
	url: Vec<String>
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Profile {
	/// unique name/identifier for this profile
	name: String,
	/// args which should be passed to yt-dl
	args: Vec<String>,
	/// If true `--download-archive DOWNLOADNAME-PROFILENAME.txt` is added to the args,
	/// where `PROFILENAME`` is the `name` field of this struct and `DOWNLOADNAME` is `name` entry of the [Download] struct.
	/// If false you can still use download archive by manual adding them to [Profile] args field.
	#[serde(default = "default_true")]
	archive: bool
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Config {
	/// path os yt-dlp binary (default: `yt-dlp`)
	#[serde(default = "default_bin_name")]
	bin_name: String,
	/// Intervall in which the programm should wait before check for downloads again in seconds,
	/// messured from start to start.
	/// The program will always wait at least 2 minutes before checking for dowload again.
	#[serde(default = "default_23h_in_seconds")]
	interval: u64,
	// Profile which is used to download the video.
	// Array is also supported, so you can download it with differnet settings/profiles (as example as audio and video)
	profile: Vec<Profile>,
	download: Vec<Download>
}

fn default_bin_name() -> String {
	"yt-dlp".into()
}

fn default_23h_in_seconds() -> u64 {
	82800
}

fn default_true() -> bool {
	true
}

fn main() {
	loop {
		let start_time = Instant::now();
		let intervall = match run() {
			Err(err) => {
				eprintln!("{err}");
				300
			},
			Ok(value) => value
		};
		let duration = start_time.elapsed();
		println!(
			"process download in {} minutes and {} seconds",
			duration.as_secs() / 60,
			duration.as_secs() % 60
		);
		let wait_time = (intervall - duration.as_secs()).max(120);
		println!("next download in {} minutes", wait_time / 60);
		sleep(Duration::from_secs(wait_time));
		println!("\n\n\n\n\n\n");
	}
}

fn run() -> anyhow::Result<u64> {
	let config: Config = basic_toml::from_str(&fs::read_to_string("config.toml")?)?;
	let mut profiles = HashMap::with_capacity(config.profile.len());

	// convert profile to hashmap
	for profile in &config.profile {
		if profiles.insert(profile.name.clone(), profile).is_some() {
			bail!("duplicate profile name {} at config", profile.name)
		}
	}

	// check if all profile refs are valid
	for download_config in &config.download {
		for profile_name in &download_config.profile {
			profiles.get(profile_name).with_context(|| {
				format!(
					"can not find profile {:?} at download {:?}",
					profile_name, download_config.name
				)
			})?;
		}
	}

	// download
	let mut errors = Vec::new();
	for download_config in &config.download {
		for profile_name in &download_config.profile {
			let profile = profiles.get(profile_name).unwrap();
			let res = download(&config, download_config, profile).with_context(|| {
				format!(
					"Falied to process {:?} with profile {:?}",
					download_config.name, profile.name
				)
			});
			if let Err(err) = res {
				eprintln!("{err:?}");
				errors.push(err);
			};
			println!("\n\n\n\n\n\n")
		}
	}

	// print error again as summary
	// otherwise the user will not be able to find it at wall of text
	for error in errors {
		eprintln!("{error:?}\n");
	}
	Ok(config.interval)
}

fn download(
	config: &Config,
	download: &Download,
	profile: &Profile
) -> anyhow::Result<()> {
	println!(
		"Download {:?} with profile {:?}",
		download.name, profile.name
	);
	let mut cmd = Command::new(&config.bin_name);
	if profile.archive {
		create_dir_all("archives")
			.with_context(|| "failed to create dir \"archives\"")?;
		cmd.args([
			"--download-archive",
			&format!("archives/{}-{}.txt", download.name, profile.name)
		]);
	}
	cmd.args(&profile.args);
	cmd.args(&download.url);
	println!("run: {cmd:?}");
	let status = cmd.status().with_context(|| "failed to execute command")?;
	if !status.success() {
		bail!("command exit with error status {status}");
	}
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
