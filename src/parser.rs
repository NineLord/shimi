use std::error::Error;
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
	/// Prints verbosely about what's going on.
	#[arg(short = 'd', long = "debug", global = true, action = SetTrue)]
	pub is_debug: bool,

	#[command(subcommand)]
	pub command: SubCommands,
}

pub struct GlobalOptions {
	pub is_debug: bool
}

impl TopCommand {
	#[inline]
	#[must_use]
	fn into_split(self) -> (GlobalOptions, SubCommands) {
		(
			GlobalOptions {
				is_debug: self.is_debug
			},
			self.command
		)
	}

	#[inline]
	pub fn run(self) -> Result<(), Box<dyn Error>> {
		let (global_options, command) = self.into_split();
		command.run(global_options)
	}
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
	Git(sub_commands::git::Command)
}

impl Run for SubCommands {
	#[inline]
	fn run(self, global_options: GlobalOptions) -> Result<(), Box<dyn Error>> {
		match self {
			SubCommands::Git(command) => command.run(global_options),
		}
	}
}