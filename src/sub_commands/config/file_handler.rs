use std::{time::{SystemTime, UNIX_EPOCH}, fs, path::{Path, PathBuf}};
use anyhow::{Context, Result};
use log::{warn, info};
use ron::{ser::PrettyConfig, de::from_reader as ron_from_reader};
use tap::Tap;
use time::{OffsetDateTime, PrimitiveDateTime};
use super::structure::Config;
use crate::utils::default_value_home_dir;

pub struct FileHandler;
#[cfg(not(feature = "dry_run"))]
const CONFIG_FILE_NAME: &str = ".shimirc.ron";
#[cfg(feature = "dry_run")]
const CONFIG_FILE_NAME: &str = ".shimirc_dryrun.ron";

// Default values
impl FileHandler {
	fn get_pretty_config() -> PrettyConfig {
		PrettyConfig::new()
			.depth_limit(5)
			.enumerate_arrays(true)
	}

	/// # Errors
	/// * Failed to get the path to the home directory of the user.
	fn get_config_file_path() -> Result<PathBuf> {
		Ok(
			default_value_home_dir()?
				.tap_mut(|home_dir| home_dir.push(CONFIG_FILE_NAME))
		)
	}
}

// Read & Write
impl FileHandler {
	/// # Errors
	/// * Failed to get the path to the config file.
	/// * Failed to open the config file.
	/// * Failed to deserialize the config file.
	pub(crate) fn read(is_verbose: bool) -> Result<Option<Config>> {
		let config_path = Self::get_config_file_path()?;

		let config_file = fs::File::open(&config_path)
			.with_context(|| format!("Failed to open the config file at: {config_path:?}"))?;

		let result = ron_from_reader(config_file).map_or_else(|_| None, |mut config| {
			if let Err(error) = Self::remove_out_of_date_ttls(&mut config, is_verbose) {
				if is_verbose {
					warn!("Failed remove outdated TTLs from config: {error:?}");
				} else {
					warn!("Failed remove outdated TTLs from config");
				}
			};
			Some(config)
		});

		Ok(result)
	}

	fn remove_out_of_date_ttls(config: &mut Config, is_verbose: bool) -> Result<()> {
		let now = OffsetDateTime::now_local().context("Can't get local time")?;
		let now = PrimitiveDateTime::new(now.date(), now.time());

		let previous_len = config.orchestration.aliases.len();
		config.orchestration.aliases.retain(|_, container_name| {
			container_name.ttl.is_none_or(|ttl| ttl >= now)
		});

		if previous_len == config.orchestration.aliases.len() {
			return Ok(()); // None of the TTLs were removed.
		}

		Self::save(config, false, is_verbose)
	}

	/// # Errors
	/// * Failed to get the path to the config file.
	/// * Failed to open the config file with write permissions.
	/// * Failed to serialize the config.
	pub(super) fn save(config: &Config, is_fail_to_parse_config: bool, is_verbose: bool) -> Result<()> {
		let config_path = Self::get_config_file_path()?;

		if is_fail_to_parse_config {
			let prev_config_path = Self::backup_prev_config(&config_path)?;
			warn!("Previous config file was backed up due to failure to parse it at: {prev_config_path:?}");
		}

		let config_file = fs::OpenOptions::new()
			.create(true)
			.truncate(true)
			.write(true)
			.open(&config_path)
			.with_context(|| format!("failed to open the config file with write permissions at: {config_path:?}"))?;

		ron::Options::default()
			.to_io_writer_pretty(&config_file, &config, Self::get_pretty_config())
			.with_context(|| format!("Failed to write the config file at: {config_path:?}"))?;

		if is_verbose {
			info!("Successfully written the config file to: {config_path:?}");
		} else {
			info!("Config file updated");
		}
		Ok(())
	}

	fn backup_prev_config(config_path: &PathBuf) -> Result<PathBuf> {
		let new_config_file_name = {
			let config_file_name = Path::new(CONFIG_FILE_NAME);
			let config_file_basename = config_file_name.file_stem().and_then(|x| x.to_str()).unwrap();
			let config_file_extension = config_file_name.extension().and_then(|x| x.to_str()).unwrap();
			format!("{config_file_basename}_{}.{config_file_extension}",
				SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis())
		};
		
		let mut new_config_path = config_path.clone();
		new_config_path.pop(); // Removes `CONFIG_FILE_NAME`
		new_config_path.push(new_config_file_name);

		fs::rename(config_path, &new_config_path)?;

		Ok(new_config_path)
	}
}
