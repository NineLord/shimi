use std::{fs, path::PathBuf};
use anyhow::{Context, Result};
use clap::Args;
use log::info;
use ron::{ser::PrettyConfig, de::from_reader};
use serde::{Serialize, Deserialize};
use tap::Tap;
use crate::{Run, GlobalOptions, utils::default_value_home_dir};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub orchestration: Orchestration,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Orchestration {
	DockerCompose,
	Kubernetes {
		name_space: String,
	},
}

/// Modify the default behavior of the script.
#[derive(Args, Debug)]
pub struct Command;

// Default values
impl Command {
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
impl Command {
	/// # Errors
	/// * Failed to get the path to the config file.
	/// * Failed to open the config file.
	/// * Failed to deserialize the config file.
	pub fn read() -> Result<Config> {
		let config_path = Self::get_config_file_path()?;

		let config_file = fs::File::open(config_path)
			.context("Failed to open the config file")?;

		let config: Config = from_reader(config_file)
			.context("Failed to deserialize the config file")?;
		Ok(config)
	}

	/// # Errors
	/// * Failed to get the path to the config file.
	/// * Failed to open the config file with write permissions.
	/// * Failed to serialize the config.
	fn save(config: &Config) -> Result<()> {
		let config_path = Self::get_config_file_path()?;

		let config_file = fs::OpenOptions::new()
			.truncate(true)
			.write(true)
			.open(&config_path)?;

		ron::Options::default()
			.to_io_writer_pretty(&config_file, &config, Self::get_pretty_config())
			.context("Failed to write the config file")?;

		info!("Successfully written the config file to: {config_path:?}");
		Ok(())
	}
}

impl Run for Command {
	fn run(self, _global_options: GlobalOptions) -> Result<()> {
		let config = Config { orchestration: Orchestration::Kubernetes { name_space: String::from("bob") } };
		Self::save(&config)?;
		/*
		 * Shaked-TODO:
		 * 1. Add to main code that tries to load the config file.
		 * 2. then add it to the GlobalOptions.
		 * 3. Learn how to do interactive terminal to edit the config file.
		 * 4. Continue with the docker commands
		 */
		Ok(())
	}
}