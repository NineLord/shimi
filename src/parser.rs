use anyhow::Result;
use clap::{Parser, Args, Subcommand, ArgAction::SetTrue};
use crate::Run;
use super::sub_commands;

/// Common shortcuts for developers.
#[derive(Parser, Debug)]
#[command(name = "s", bin_name = "s")]
#[command(about)]
#[command(long_about = "A Script of common things a developer might need.
It contains commands that are too incontinent to type every time,
or just hard to remember.")]
#[command(version)]
pub struct TopCommand {
	/// Prints extra information about what happens at run time.
	#[arg(short = 'v', long = "verbose", global = true, action = SetTrue)]
	pub is_verbose: bool,

	#[command(subcommand)]
	pub command: SubCommands,
}

pub struct GlobalOptions {
	pub is_verbose: bool
}

impl TopCommand {
	#[inline]
	#[must_use]
	pub fn into_split(self) -> (GlobalOptions, SubCommands) {
		(
			GlobalOptions {
				is_verbose: self.is_verbose
			},
			self.command
		)
	}
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
	Git(sub_commands::git::Command)
}

impl Run for SubCommands {
	#[inline]
	fn run(self, global_options: GlobalOptions) -> Result<()> {
		match self {
			Self::Git(command) => command.run(global_options),
		}
	}
}