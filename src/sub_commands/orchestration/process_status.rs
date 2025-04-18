use anyhow::Result;
use clap::{Args, ArgAction::SetTrue};
use super::{command::RunOrchestration, global_orch_options::GlobalOrchOptions};
use crate::{utils::RunCommand, sub_commands::config::OrchestrationType};

/// Shows the current state of your orchestration
#[derive(Args, Debug)]
#[command(visible_alias = "ps")]
pub struct Arguments {
	/// Will display only the container/pods names
	#[arg(long = "only-names", action = SetTrue)]
	pub is_names_only: bool,

	#[arg(short = 'a', long = "all", action = SetTrue,
	help = "Show all containers/pods",
	long_help = "Show all containers/pods.
In docker-compose it will include not running containers.
In kubernetes it will include all namespaces.")]
	pub is_show_all: bool,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		match global_options.get_orchestration_type() {
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
					arguments.push("--no-headers");
					arguments.push("--output");
					arguments.push("custom-columns=NAME:.metadata.name");
				}

				if self.is_show_all {
					arguments.push("--all-namespaces");
				} else {
					global_options.add_name_space(&mut arguments);
				}

				RunCommand::exec_with_args("oc", arguments)
			},
		}
	}
}

impl Arguments {
	pub fn get_container_names(global_orch_options: &GlobalOrchOptions, is_show_all: bool) -> Result<Vec<String>> {
		let output = match global_orch_options.get_orchestration_type() {
			OrchestrationType::DockerCompose => {
				let mut arguments = vec!["container", "ls", "--format", "{{.Names}}"];
				if is_show_all {
					arguments.push("--all");
				}
				RunCommand::run_with_args_sync("docker", arguments)
			},
			OrchestrationType::Kubernetes => {
				let mut arguments = vec!["get", "pods", "--no-headers", "--output", "custom-columns=NAME:.metadata.name"];

				if is_show_all {
					arguments.push("--all-namespaces");
				} else {
					global_orch_options.add_name_space(&mut arguments);
				}

				RunCommand::run_with_args_sync("oc", arguments)
			},
		}?;

		let container_names = if cfg!(feature = "dry_run") {
			vec![
				String::from("avatar"),
				String::from("you_tube_2"),
				String::from("path-of-exile-1"),
				String::from("you_tube_1"),
				String::from("last-epoch"),
				String::from("you_tube_3"),
				String::from("path-of-exile-2"),
				String::from("spongebob"),
			]
		} else {
			std::str::from_utf8(&output.stdout)?
				.split('\n')
				.filter(|container_name| !container_name.is_empty())
				.map(String::from)
				.collect::<Vec<String>>()
		};
		
		Ok(container_names)
	}
}