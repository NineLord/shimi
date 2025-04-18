use anyhow::Result;
use clap::Args;
use super::{command::RunOrchestration, global_orch_options::GlobalOrchOptions};
use crate::{utils::RunCommand, sub_commands::config::OrchestrationType};

const DEFAULT_COMMAND: &str = "/bin/bash";

/// Run a command in a running container
#[derive(Args, Debug)]
#[command(visible_aliases = ["x", "exec", "ent", "enter"])]
pub struct Arguments {
	/// The name of the container that going to execute the command
	pub container_name: String,

	/// Overwrite the default command
	#[arg(short, long, default_value = DEFAULT_COMMAND)]
	pub command: String,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		let container_name = global_options.get_container_name(&self.container_name)?;

		match global_options.get_orchestration_type() {
			OrchestrationType::DockerCompose => {
				RunCommand::exec_with_args("docker", ["exec", "--interactive", "--tty", &container_name, &self.command])
			},
			OrchestrationType::Kubernetes => {
				let mut arguments = vec!["exec", "--stdin", "--tty"];
				global_options.add_name_space(&mut arguments);
				arguments.push(&container_name);
				arguments.push(&self.command);
				RunCommand::exec_with_args("oc", arguments)
			},
		}
	}
}
