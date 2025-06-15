use std::iter;
use anyhow::Result;
use clap::{Args, Subcommand};
use strum::{VariantArray, IntoStaticStr, EnumDiscriminants};
use crate::{commands::{GetSubCommandAliases, GetSubCommandNames, GetSubCommandsNames, Run}, sub_commands::config::OrchestrationType, GlobalOptions};
use super::{global_orch_options::GlobalOrchOptions, process_status, execute, logs, ip, port_forward, up, down, reset};

const VISIBLE_ALIASES: [&str ; 5] = ["orch", "d", "docker", "kub", "kubernetes"];

pub trait RunOrchestration : Sized {
	/// # Errors
	/// Should return an error with explanation why the command couldn't run.
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()>;
}

/// Includes sub-commands related to docker/docker-compose/kubernetes.
#[derive(Args, Debug)]
#[command(visible_aliases = VISIBLE_ALIASES)]
pub struct Command {
	#[arg(long = "type", global = true, value_enum,
	help_heading = Some("Orchestration's Options"),
	help = "Overwrite the orchestration type taken from the config")]
	pub orchestration_type: Option<OrchestrationType>,

	#[arg(short = 'n', long = "namespace", global = true,
	help_heading = Some("Orchestration's Options"),
	help = "Overwrite the namespace taken from the config",
	long_help = "Overwrite the namespace taken from the config.
Will be effective only when 'kubernetes' is the chosen orchestration."
)]
	pub name_space: Option<String>,

	#[command(subcommand)]
	pub command: CommandInner,
}

#[derive(Subcommand, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(IntoStaticStr, VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum CommandInner {
	ProcessStatus(process_status::Arguments),
	Execute(execute::Arguments),
	Logs(logs::Arguments),
	Ip(ip::Arguments),
	PortForward(port_forward::Arguments),
	Up(up::Arguments),
	Down(down::Arguments),
	Reset(reset::Arguments),
}

impl GetSubCommandAliases for Command {
	fn get_sub_command_aliases() -> &'static [&'static str] {
		&VISIBLE_ALIASES
	}
}

impl GetSubCommandNames for CommandInnerDiscriminants {
	fn get_sub_command_names(self) -> impl Iterator<Item = &'static str> {
		let command: &'static str = self.into();
		let command = iter::once(command);
		match self {
			Self::ProcessStatus => process_status::Arguments::get_sub_command_aliases(),
			Self::Execute => execute::Arguments::get_sub_command_aliases(),
			Self::Logs => logs::Arguments::get_sub_command_aliases(),
			Self::Ip => ip::Arguments::get_sub_command_aliases(),
			Self::PortForward => port_forward::Arguments::get_sub_command_aliases(),
			Self::Up => up::Arguments::get_sub_command_aliases(),
			Self::Down => down::Arguments::get_sub_command_aliases(),
			Self::Reset => reset::Arguments::get_sub_command_aliases(),
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
		let Self { orchestration_type, name_space, command } = self;
		let global_orch_options = GlobalOrchOptions::new(global_options, orchestration_type, name_space);
		match command {
			CommandInner::ProcessStatus(command) => command.run_orch(&global_orch_options),
			CommandInner::Execute(command) => command.run_orch(&global_orch_options),
			CommandInner::Logs(command) => command.run_orch(&global_orch_options),
			CommandInner::Ip(command) => command.run_orch(&global_orch_options),
			CommandInner::PortForward(command) => command.run_orch(&global_orch_options),
			CommandInner::Up(command) => command.run_orch(&global_orch_options),
			CommandInner::Down(command) => command.run_orch(&global_orch_options),
			CommandInner::Reset(command) => command.run_orch(&global_orch_options),
		}
	}
}
