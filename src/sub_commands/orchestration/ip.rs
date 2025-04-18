use anyhow::Result;
use clap::Args;
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch}};
use crate::{utils::RunCommand, sub_commands::config::OrchestrationType};

/// Gets the container's IP
#[derive(Args, Debug)]
pub struct Arguments {
	/// The name of the container that going to give his IP
	pub container_name: String,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		const IS_EXACT_MATCH: bool = false; // Shaked-TODO: receive it as optional argument
		let container_name = {
			let options = if IS_EXACT_MATCH {
				IsExactMatch::Yes
			} else {
				IsExactMatch::No(IsTryGetMatch::default())
			};
			global_options.get_container_name(&self.container_name, options)?
		};

		let _output = match global_options.get_orchestration_type() {
			OrchestrationType::DockerCompose => {
				RunCommand::run_with_args_sync("docker", ["inspect", &container_name])
			},
			OrchestrationType::Kubernetes => {
				let mut arguments = vec!["TODO", &container_name];
				global_options.add_name_space(&mut arguments);
				RunCommand::run_with_args_sync("oc", arguments)
			},
		}?;

		todo!("Either parse the given output or display it right away with exec instead of run_sync")
	}
}
