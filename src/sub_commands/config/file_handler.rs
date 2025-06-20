use std::{time::{SystemTime, UNIX_EPOCH}, fs, path::PathBuf};
use anyhow::{Context, Result};
use log::{warn, info};
use time::{OffsetDateTime, PrimitiveDateTime};
use super::structure::Config;
use crate::prelude::PACKAGE_NAME;

pub struct FileHandler;
#[cfg(not(feature = "dry_run"))]
const CONFIG_FILE_NAME: &str = "configurations";
#[cfg(feature = "dry_run")]
const CONFIG_FILE_NAME: &str = "configurations_dryrun";

pub struct ReadResult {
	pub config: Config,
	pub is_fail_to_parse_config: bool,
}

// Read & Write
impl FileHandler {
	pub fn read(is_verbose: bool) -> ReadResult {
		let mut is_fail_to_parse_config = false;
		let mut config: Config = match confy::load(PACKAGE_NAME, CONFIG_FILE_NAME) {
			Ok(config) => config,
			Err(error) => {
				if is_verbose {
					warn!("Failed to read the config, continuing with default config: {error}");
				} else {
					warn!("Failed to read the config, continuing with default config");
				}
				is_fail_to_parse_config = true;
				Config::default()
			},
		};
		if let Err(error) = Self::remove_out_of_date_ttls(&mut config, is_verbose) {
			if is_verbose {
				warn!("Failed remove outdated TTLs from config: {error:?}");
			} else {
				warn!("Failed remove outdated TTLs from config");
			}
		};
		ReadResult { config, is_fail_to_parse_config }
	}

	fn remove_out_of_date_ttls(config: &mut Config, is_verbose: bool) -> Result<()> {
		let now = OffsetDateTime::now_local().context("Can't get local time")?;
		let now = PrimitiveDateTime::new(now.date(), now.time());

		let previous_len = config.orchestration().aliases.len();
		config.orchestration_mut().aliases.retain(|_, container_name| {
			container_name.ttl.is_none_or(|ttl| ttl >= now)
		});

		if previous_len == config.orchestration().aliases.len() {
			return Ok(()); // None of the TTLs were removed.
		}

		Self::save(config, false, is_verbose)
	}

	#[cfg(feature = "dry_run")]
	#[allow(dead_code)]
	pub fn get_path() -> Result<PathBuf> {
		Self::get_configuration_file_path()
	}

	fn get_configuration_file_path() -> Result<PathBuf> {
		Ok(confy::get_configuration_file_path(PACKAGE_NAME, CONFIG_FILE_NAME)?)
	}

	/// # Errors
	/// * Failed to get the path to the config file.
	/// * Failed to open the config file with write permissions.
	/// * Failed to serialize the config.
	pub(super) fn save(config: &Config, is_fail_to_parse_config: bool, is_verbose: bool) -> Result<()> {
		let config_path = Self::get_configuration_file_path()?;

		if is_fail_to_parse_config {
			let prev_config_path = Self::backup_prev_config(&config_path)?;
			warn!("Previous config file was backed up due to failure to parse it at: {prev_config_path:?}");
		}

		confy::store(PACKAGE_NAME, CONFIG_FILE_NAME, config)?;

		if is_verbose {
			info!("Successfully written the config file to: {config_path:?}");
		} else {
			info!("Config file updated");
		}

		Ok(())
	}

	fn backup_prev_config(config_path: &PathBuf) -> Result<PathBuf> {
		let new_basename = {
			let mut basename = config_path
				.file_stem()
				.expect("The given config_path expected to have filename in it already")
				.to_owned();
			basename.push(format!("_backup_{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()));
			basename
		};
		let previous_extension = config_path.extension().unwrap();
		
		let mut new_config_path = config_path.clone();
		new_config_path.set_file_name(new_basename);
		new_config_path.set_extension(previous_extension);

		fs::rename(config_path, &new_config_path)?;

		Ok(new_config_path)
	}
}
