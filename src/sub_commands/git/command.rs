use anyhow::Result;
use clap::{Args, Subcommand};
use crate::{Run, GlobalOptions};
use super::{global_ignore, global_user_email, ssh_key_gen};

/// Includes sub-commands related to git.
#[derive(Args, Debug)]
pub struct Command {
	#[command(subcommand)]
	pub command: CommandInner,
}

#[derive(Subcommand, Debug)]
pub enum CommandInner {
	GlobalIgnore(global_ignore::Arguments),
	GlobalUserEmail(global_user_email::Arguments),
	SshKeyGenerator(ssh_key_gen::Arguments),
}

impl Run for Command {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		match self.command {
			CommandInner::GlobalIgnore(command) => command.run(global_options),
			CommandInner::GlobalUserEmail(command) => command.run(global_options),
			CommandInner::SshKeyGenerator(command) => command.run(global_options),
		}
	}
}