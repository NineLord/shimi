use anyhow::Result;
use clap::{Args, ArgAction::SetTrue};
use super::command::{RunOrchestration, GlobalOrchOptions};
use crate::{GlobalOptions, utils::{RunCommand, ExitError}, sub_commands::config::OrchestrationType};

/// Shows the current state of your orchestration
#[derive(Args, Debug)]
#[command(visible_alias = "ps")]
pub struct Arguments {
	/// Will display only the container/pods names
	#[arg(long = "only-names", action = SetTrue)]
	pub is_names_only: bool,

	// / Show all containers/pods (default shows only running)
	#[arg(short = 'a', long = "all", action = SetTrue,
	help = "Show all containers/pods",
	long_help = "Show all containers/pods.
In docker-compose it will include not running containers.
In kubernetes it will include all namespaces.")]
	pub is_show_all: bool,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, _global_options: &GlobalOptions, global_orch_options: &GlobalOrchOptions) -> Result<()> {
		match global_orch_options.get_orchestration_type() {
			OrchestrationType::DockerCompose => {
				let mut arguments = if self.is_names_only {
					vec!["container", "ls", "--format", "{{.Names}}"]
				} else {
					vec!["ps", "--format", "table {{.ID}}  {{.Names}}\t{{.Status}}"]
				};
				if self.is_show_all {
					arguments.push("--all");
				}
				RunCommand::exec_with_args("docker", arguments)
			},
			OrchestrationType::Kubernetes => {
				let mut arguments = vec!["get", "pods"];
				if self.is_names_only {
					arguments.push("--output");
					arguments.push("custom-columns=NAME:.metadata.name");
				}

				if self.is_show_all {
					arguments.push("--all-namespaces");
				} else {
					let name_space = match global_orch_options.get_name_space() {
						Ok(name_space) => name_space,
						Err(error) => ExitError::BadArgument.exit(error),
					};

					arguments.push("--namespace");
					arguments.push(name_space);
				}

				RunCommand::exec_with_args("oc", arguments)
			},
		}
	}
}