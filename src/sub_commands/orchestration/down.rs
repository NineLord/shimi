use anyhow::Result;
use clap::{Args, ArgAction::SetFalse};
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch}};
use crate::{utils::RunCommand, sub_commands::config::OrchestrationType};

/// Stop running container(s)
#[derive(Args, Debug)]
#[command(visible_aliases = ["stop", "remove", "rm"])]
pub struct Arguments {
	/// The name of the container that going to stop running.
	/// If not given, will stop all the containers.
	pub container_name: Option<String>,

	#[arg(long = "no-remove", action = SetFalse,
	help = "Remove a container instead of stopping it",
	long_help = "Remove a container instead of stopping it.
Will be effective only when 'docker-compose' is the chosen orchestration.")]
	pub is_remove: bool,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		match &self.container_name {
			Some(container_name) => stop(global_options, container_name, self.is_remove),
			None => stop_all(global_options, self.is_remove),
		}
	}
}

pub fn stop(global_options: &GlobalOrchOptions, container_name: &str, is_remove: bool) -> Result<()> {
	const IS_EXACT_MATCH: bool = false; // Shaked-TODO: receive it as optional argument.
	let container_name = {
		let options = if IS_EXACT_MATCH {
			IsExactMatch::Yes
		} else {
			let is_all_containers = match global_options.get_orchestration_type() {
				OrchestrationType::DockerCompose => true,
				OrchestrationType::Kubernetes => false,
			};
			IsExactMatch::No(IsTryGetMatch::Yes { is_must_match: true, is_all_containers })
		};
		global_options.get_container_name(container_name, options)?
	};

	match global_options.get_orchestration_type() {
		OrchestrationType::DockerCompose => {
			if is_remove {
				RunCommand::exec_with_args("docker", ["rm", &container_name])
			} else {
				RunCommand::exec_with_args("docker", ["stop", &container_name])
			}
		},
		OrchestrationType::Kubernetes => {
			let mut arguments = vec!["TODO", &container_name];
			global_options.add_name_space(&mut arguments);
			RunCommand::exec_with_args("oc", arguments)
		},
	}
}

pub fn stop_all(global_options: &GlobalOrchOptions, _is_remove: bool) -> Result<()> {
	match global_options.get_orchestration_type() {
		OrchestrationType::DockerCompose => {
			RunCommand::exec_with_args("docker", ["up"])
		},
		OrchestrationType::Kubernetes => {
			let mut arguments = vec!["TODO"];
			global_options.add_name_space(&mut arguments);
			RunCommand::exec_with_args("oc", arguments)
		},
	}
}
