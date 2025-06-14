use std::iter;
use anyhow::Result;
use clap::{Args, Subcommand};
use strum::{VariantArray, IntoStaticStr, EnumDiscriminants};
use crate::{commands::{GetSubCommandAliases, GetSubCommandNames, GetSubCommandsNames, Run}, GlobalOptions};
use super::{global_ignore, global_user_email, ssh_key_gen};

/// Includes sub-commands related to git.
#[derive(Args, Debug)]
pub struct Command {
	#[command(subcommand)]
	pub command: CommandInner,
}

#[derive(Subcommand, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(IntoStaticStr, VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CommandInner {
	GlobalIgnore(global_ignore::Arguments),
	GlobalUserEmail(global_user_email::Arguments),
	SshKeyGenerator(ssh_key_gen::Arguments),
}

impl GetSubCommandAliases for Command {
	fn get_sub_command_aliases() -> &'static [&'static str] {
		const VISIBLE_ALIASES: [&str ; 0] = [];
		&VISIBLE_ALIASES
	}
}

impl GetSubCommandNames for CommandInnerDiscriminants {
	fn get_sub_command_names(self) -> impl Iterator<Item = &'static str> {
		let command: &'static str = self.into();
		let command = iter::once(command);
		match self {
			Self::GlobalIgnore => global_ignore::Arguments::get_sub_command_aliases(),
			Self::GlobalUserEmail => global_user_email::Arguments::get_sub_command_aliases(),
			Self::SshKeyGenerator => ssh_key_gen::Arguments::get_sub_command_aliases(),
		}
			.iter()
			.copied()
			.chain(command)
	}
}

impl GetSubCommandsNames for Command {
	fn get_sub_commands_names() -> impl Iterator<Item = &'static str> {
		CommandInnerDiscriminants::VARIANTS
			.iter()
			.flat_map(|sub_command| sub_command.get_sub_command_names())
	}
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