use std::error::Error;
use clap::{Args, Subcommand};
use crate::{Run, GlobalOptions};
use super::ssh_key_gen;

/// Includes sub-commands related to git.
#[derive(Args, Debug)]
#[command(visible_alias = "g")]
pub struct Command {
	#[command(subcommand)]
	pub command: CommandInner,
}

#[derive(Subcommand, Debug)]
pub enum CommandInner {
	SshKeyGenerator(ssh_key_gen::Arguments),
}

impl Run for Command {
	#[inline]
	fn run(self, global_options: GlobalOptions) -> Result<(), Box<dyn Error>> {
		match self.command {
			CommandInner::SshKeyGenerator(command) => command.run(global_options),
		}
	}
}