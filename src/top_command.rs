use anyhow::Result;
use clap::{Parser, Subcommand, ArgAction::SetTrue};
use crate::{sub_commands::{self, config::{self, Config}}, Run};

/// Common shortcuts for developers.
#[derive(Parser, Debug)]
#[command(name = "s", bin_name = "s")]
#[command(about)]
#[command(long_about = "A Script of common things a developer might need.
It contains commands that are too inconvenient to type every time,
or just hard to remember.")]
#[command(version)]
pub struct TopCommand {
	// Prints extra information about what happens at run time (lowers the logger to TRACE).
	#[arg(short = 'v', long = "verbose", global = true, action = SetTrue,
		help = "Prints extra information about what happens at run time",
		long_help = "Prints extra information about what happens at run time.
In addition it changes the logger to TRACE.
You are able to change the logger level to any level (error/info/warn/debug/trace) using the environment variable `SHIMI_LOG`.
The environment variable has higher priority to this flag."
	)]
	pub is_verbose: bool,

	#[command(subcommand)]
	pub command: SubCommands,
}

pub struct GlobalOptions {
	pub is_verbose: bool,
	pub config: Config,
}

impl TopCommand {
	#[inline]
	#[must_use]
	pub fn into_split(self) -> (GlobalOptions, SubCommands) {
		(
			GlobalOptions {
				is_verbose: self.is_verbose,
				config: config::Command::read().unwrap_or_default(),
			},
			self.command
		)
	}
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
	Git(sub_commands::git::Command),
	Config(sub_commands::config::Command),
}

impl Run for SubCommands {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		match self {
			Self::Git(command) => command.run(global_options),
			Self::Config(command) => command.run(global_options),
		}
	}
}