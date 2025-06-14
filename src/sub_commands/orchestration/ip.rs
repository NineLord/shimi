use anyhow::Result;
use clap::{Args, ArgAction::SetTrue};
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch}};
use crate::{sub_commands::config::OrchestrationType, utils::{ExitError, RunCommand}};

/// Gets the container's IP
#[derive(Args, Debug)]
pub struct Arguments {
	/// The name of the container that going to give his IP
	pub container_name: String,

	/// If true, won't try to convert the container name to his alias.
	#[arg(short = 'e', long = "exact-match", action = SetTrue)]
	pub is_exact_match: bool,
}

impl RunOrchestration for Arguments {
	#[allow(clippy::items_after_statements, unreachable_code, unused)]
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		ExitError::NotYetImplemented.exit_with_message("ip::run_orch");
		let container_name = {
			let options = if self.is_exact_match {
				IsExactMatch::Yes
			} else {
				IsExactMatch::No(IsTryGetMatch::default())
			};
			global_options.get_container_name(self.container_name, options)?
		};

		let _output = match global_options.get_orchestration_type() {
			OrchestrationType::DockerCompose => {
				RunCommand::run_with_args_sync("docker", ["inspect", &container_name])
			},
			OrchestrationType::Kubernetes => {
				let mut arguments = vec!["no idea", &container_name];
				global_options.add_name_space(&mut arguments);
				RunCommand::run_with_args_sync("oc", arguments)
			},
		}?;

		// Either parse the given output or display it right away with exec instead of run_sync
	}
}
