use std::{fs, path::PathBuf};
use anyhow::{Context, Result};
use log::{info, warn};
use ron::{ser::PrettyConfig, de::from_reader};
use tap::Tap;
use super::structure::Config;
use crate::{GlobalOptions, utils::default_value_home_dir};

pub struct FileHandler;

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
		const CONFIG_FILE_NAME: &str = ".shimirc.ron";

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
	pub(crate) fn read(version: &str) -> Result<Config> {
		let config_path = Self::get_config_file_path()?;

		let config_file = fs::File::open(&config_path)
			.with_context(|| format!("Failed to open the config file at: {config_path:?}"))?;

		let config: Config = from_reader(config_file)
			.with_context(|| {
				warn!("Failed to parse previous config file.
could it be from previous versions of the tool? (Current version: {version:?})
Continuing with default config."); // No backward support as of yet.
				format!("Failed to deserialize the config file at: {config_path:?}")
			})?;

		Ok(config)
	}

	/// # Errors
	/// * Failed to get the path to the config file.
	/// * Failed to open the config file with write permissions.
	/// * Failed to serialize the config.
	pub(super) fn save(config: &Config, global_options: &GlobalOptions) -> Result<()> {
		let config_path = Self::get_config_file_path()?;

		if !cfg!(feature = "dry_run") {
			let config_file = fs::OpenOptions::new()
				.create(true)
				.truncate(true)
				.write(true)
				.open(&config_path)
				.with_context(|| format!("failed to open the config file with write permissions at: {config_path:?}"))?;
	
			ron::Options::default()
				.to_io_writer_pretty(&config_file, &config, Self::get_pretty_config())
				.with_context(|| format!("Failed to write the config file at: {config_path:?}"))?;
		}

		if global_options.is_verbose {
			info!("Successfully written the config file to: {config_path:?}");
		} else {
			info!("Config file updated");
		}
		Ok(())
	}
}
