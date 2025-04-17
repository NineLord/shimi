use anyhow::Result;
use clap::{Args, Subcommand};
use crate::{Run, GlobalOptions, sub_commands::config::OrchestrationType};
use super::{global_orch_options::GlobalOrchOptions, process_status, execute};

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

#[derive(Subcommand, Debug)]
pub enum CommandInner {
	ProcessStatus(process_status::Arguments),
	Execute(execute::Arguments),
}

impl Run for Command {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		let Self { orchestration_type, name_space, command } = self;
		let global_orch_options = GlobalOrchOptions::new(&global_options.config, orchestration_type, name_space);
		match command {
			CommandInner::ProcessStatus(command) => command.run_orch(global_options, &global_orch_options),
			CommandInner::Execute(command) => command.run_orch(global_options, &global_orch_options),
		}
	}
}