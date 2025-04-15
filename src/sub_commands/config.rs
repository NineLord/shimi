use std::{fs, path::PathBuf};
use anyhow::{Context, Result};
use clap::{Args, ArgAction::SetTrue};
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

impl Default for Config {
	fn default() -> Self {
		Self {
			orchestration: Orchestration::DockerCompose
		}
	}
}

#[derive(Args, Debug)]
#[command(about = "Modify the default behavior of the script.")]
#[command(long_about = "Modify the default behavior of the script.
Will enter into interactive CLI that will allow you to edit your config file.")]
pub struct Command {
	#[arg(short = 'w', long = "wizard", action = SetTrue,
		help = "If turned on, will start a wizard that will go through all the configurations.",
		long_help = "If turned on, will start a wizard that will go through all the configurations,
and allow you to edit each one of them."
	)]
	pub is_wizard: bool,
}

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

		let config_file = fs::File::open(&config_path)
			.with_context(|| format!("Failed to open the config file at: {config_path:?}"))?;

		let config: Config = from_reader(config_file)
			.with_context(|| format!("Failed to deserialize the config file at: {config_path:?}"))?;

		Ok(config)
	}

	/// # Errors
	/// * Failed to get the path to the config file.
	/// * Failed to open the config file with write permissions.
	/// * Failed to serialize the config.
	fn save(config: &Config) -> Result<()> {
		let config_path = Self::get_config_file_path()?;

		let config_file = fs::OpenOptions::new()
			.create(true)
			.truncate(true)
			.write(true)
			.open(&config_path)
			.with_context(|| format!("failed to open the config file with write permissions at: {config_path:?}"))?;

		ron::Options::default()
			.to_io_writer_pretty(&config_file, &config, Self::get_pretty_config())
			.with_context(|| format!("Failed to write the config file at: {config_path:?}"))?;

		info!("Successfully written the config file to: {config_path:?}");
		Ok(())
	}
}

// Wizard
impl Command {
	fn wizard(global_options: &GlobalOptions) -> Option<Config> {
		let GlobalOptions { is_verbose: _, config } = global_options;

		todo!()
	}
}

// Manual
impl Command {
	fn manual(global_options: &GlobalOptions) -> Option<Config> {
		unimplemented!()
	}
}

impl Run for Command {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		let config = if self.is_wizard {
			Self::wizard(global_options)
		} else {
			Self::manual(global_options)
		};

		match config {
			Some(config) => Self::save(&config)?,
			None => info!("Aborted."),
		}

		Ok(())
	}
}