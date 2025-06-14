use anyhow::Result;
use clap::{Args, ArgAction::SetTrue};
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch}};
use crate::{sub_commands::config::OrchestrationType, utils::{ExitError, RunCommand}, commands::GetSubCommandAliases};

const VISIBLE_ALIASES: [&str ; 3] = ["pf", "export", "ex"];

/// Expose the port of a given container to your local machine.
#[derive(Args, Debug)]
#[command(visible_aliases = VISIBLE_ALIASES)]
pub struct Arguments {
	/// The name of the container that going to expose his port.
	pub container_name: String,

	/// The port that going to exposed.
	pub from: usize,

	/// The port that going to be opened on your local machine.
	pub to: usize,

	/// If true, won't try to convert the container name to his alias.
	#[arg(short = 'e', long = "exact-match", action = SetTrue)]
	pub is_exact_match: bool,
}

impl GetSubCommandAliases for Arguments {
	fn get_sub_command_aliases() -> &'static [&'static str] {
		&VISIBLE_ALIASES
	}
}

impl RunOrchestration for Arguments {
	#[allow(clippy::items_after_statements, unreachable_code, unused)]
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		ExitError::NotYetImplemented.exit_with_message("port_forward::run_orch");
		let container_name = {
			let options = if self.is_exact_match {
				IsExactMatch::Yes
			} else {
				IsExactMatch::No(IsTryGetMatch::default())
			};
			global_options.get_container_name(self.container_name, options)?
		};

		match global_options.get_orchestration_type() {
			OrchestrationType::DockerCompose => {
				RunCommand::exec_with_args("docker", ["no idea", &container_name])
			},
			OrchestrationType::Kubernetes => {
				let mut arguments = vec!["no idea", &container_name];
				global_options.add_name_space(&mut arguments);
				RunCommand::exec_with_args("oc", arguments)
			},
		}
	}
}
