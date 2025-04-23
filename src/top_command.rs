use std::process;
use anyhow::Result;
use clap::{ArgAction::SetTrue, CommandFactory, FromArgMatches, Parser, Subcommand};
use log::{warn, error};
use crate::{logger, sub_commands::{self, config::{self, Config}}, Run};

#[allow(dead_code, clippy::needless_raw_string_hashes)]
const SHIMI: &str = r#"
   _____ __    _           _ 
  / ___// /_  (_)___ ___  (_)
  \__ \/ __ \/ / __ `__ \/ / 
 ___/ / / / / / / / / / / /  
/____/_/ /_/_/_/ /_/ /_/_/   
"#; // TODO: add this to the long help

#[derive(Parser, Debug)]
#[command(name = "s", bin_name = "s")]
#[command(about = "Common shortcuts for developers.")]
#[command(long_about = "A Script of common things a developer might need.
It contains commands that are too inconvenient to type every time,
or just hard to remember.")]
#[command(version)]
pub struct TopCommand {
	#[arg(short = 'v', long = "verbose", global = true, action = SetTrue,
		help = "Prints extra information about what happens at run time",
		long_help = "Prints extra information about what happens at run time.
In addition it changes the logger to TRACE.
You are able to change the logger level to any level (error/info/warn/debug/trace)
using the environment variable `SHIMI_LOG`.
The environment variable has higher priority to this flag."
	)]
	pub is_verbose: bool,

	#[command(subcommand)]
	pub command: SubCommands,
}

#[derive(Debug)]
pub struct GlobalOptions {
	pub is_verbose: bool,
	pub version: String,
	pub config: Config,
	pub is_fail_to_parse_config: bool,
}

impl TopCommand {
	/// # Panics
	/// If missing version at `Cargo.toml`.
	#[must_use]
	pub fn parse() -> (GlobalOptions, SubCommands) {
		let (version, top_command) = Self::parse_version();
		logger::init(top_command.is_verbose);
		let Some(version) = version else {
			error!("Shimi script missing current version number");
			process::exit(1);
		};

		let (config, is_fail_to_parse_config) = match config::FileHandler::read() {
			Ok(Some(config)) => (config, false),
			Err(_) => (Config::default(version.clone()), false),
			Ok(None) => {
				warn!("Failed to parse previous config file.
could it be from previous versions of the tool? (Current version: {version:?})
Continuing with default config."); // No backward support as of yet.
				(Config::default(version.clone()), true)
			},
		};
		(
			GlobalOptions {
				is_verbose: top_command.is_verbose,
				config,
				is_fail_to_parse_config,
				version,
			},
			top_command.command
		)
	}

	/// Modified version of [`clap_builder::derive::Parser::parse()`]
	/// that also returned the version of the command.
	fn parse_version() -> (Option<String>, Self) {
		let command = <Self as CommandFactory>::command();
		let version = command.get_version().map(String::from);
		let mut matches = command.get_matches();
        let result = <Self as FromArgMatches>::from_arg_matches_mut(&mut matches)
            .map_err(|error| {
				let mut command = <Self as CommandFactory>::command();
				error.format(&mut command)
			});
        match result {
            Ok(command) => (version, command),
            Err(error) => error.exit(),
        }
	}
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
	Git(sub_commands::git::Command),
	Orchestration(sub_commands::orchestration::Command),
	Config(sub_commands::config::Command),
}

impl Run for SubCommands {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		match self {
			Self::Git(command) => command.run(global_options),
			Self::Orchestration(command) => command.run(global_options),
			Self::Config(command) => command.run(global_options),
		}
	}
}
