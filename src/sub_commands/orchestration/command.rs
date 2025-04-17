use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use crate::{Run, GlobalOptions, sub_commands::config::{Config, OrchestrationType}};
use super::process_status;

pub trait RunOrchestration : Sized {
	/// # Errors
	/// Should return an error with explanation why the command couldn't run.
	fn run_orch(self, global_options: &GlobalOptions, global_orch_options: &GlobalOrchOptions) -> Result<()>;
}

/// Includes sub-commands related to docker/docker-compose/kubernetes.
#[derive(Args, Debug)]
#[command(visible_aliases = ["orch", "d", "docker", "kub", "kubernetes"])]
pub struct Command {
	#[arg(short = 't', long = "type", global = true, value_enum,
	help = "Overwrite the orchestration type taken from the config")]
	pub orchestration_type: Option<OrchestrationType>,

	#[arg(short = 'n', long = "namespace", global = true,
	help = "Overwrite the namespace taken from the config",
	long_help = "Overwrite the namespace taken from the config.
Will be effective only when 'kubernetes' is the chosen orchestration."
)]
	pub name_space: Option<String>,

	#[command(subcommand)]
	pub command: CommandInner,
}

#[derive(Debug)]
pub struct GlobalOrchOptions<'a> {
	config: &'a Config,
	orchestration_type: Option<OrchestrationType>,
	name_space: Option<String>,
}

impl GlobalOrchOptions<'_> {
	pub const fn get_orchestration_type(&self) -> OrchestrationType {
		match self.orchestration_type {
			Some(variant) => variant,
			None => self.config.orchestration.variant,
		}
	}

	pub fn get_name_space(&self) -> Result<&String> {
		match (&self.name_space, &self.config.orchestration.kubernetes) {
			(Some(name_space), _) => Ok(name_space),
			(None, Some(kubernetes)) => Ok(&kubernetes.name_space),
			(None, None) => Err(anyhow!("Namespace wasn't set in the config nor given via optional argument")),
		}
	}
}

#[derive(Subcommand, Debug)]
pub enum CommandInner {
	ProcessStatus(process_status::Arguments),
}

impl Run for Command {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		let Self { orchestration_type, name_space, command } = self;
		let global_orch_options = GlobalOrchOptions {
			config: &global_options.config,
			orchestration_type,
			name_space
		};
		match command {
			CommandInner::ProcessStatus(command) => command.run_orch(global_options, &global_orch_options),
		}
	}
}