use anyhow::Result;
use clap::{Args, ArgAction::SetFalse};
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch}};
use crate::{utils::RunCommand, sub_commands::config::OrchestrationType};

/// Starts running container(s)
#[derive(Args, Debug)]
#[command(visible_alias = "start")]
pub struct Arguments {
	/// The name of the container that going to start running.
	/// If not given, will start all the containers.
	pub container_name: Option<String>,

	#[arg(short = 's', long = "only-start", action = SetFalse,
	help = "Start a downed container instead of ALSO creating it",
	long_help = "Start a downed container instead of ALSO creating it.
Will be effective only when 'docker-compose' is the chosen orchestration.")]
	pub is_also_create: bool,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		match &self.container_name {
			Some(container_name) => start(global_options, container_name, self.is_also_create),
			None => start_all(global_options, self.is_also_create),
		}
	}
}

pub fn start(global_options: &GlobalOrchOptions, container_name: &str, is_also_create: bool) -> Result<()> {
	const IS_EXACT_MATCH: bool = false; // Shaked-TODO: receive it as optional argument
	let container_name = {
		let options = match (IS_EXACT_MATCH, global_options.get_orchestration_type()) {
			(true, _) => IsExactMatch::Yes,
			(false, OrchestrationType::DockerCompose) => IsExactMatch::No(IsTryGetMatch::Yes { is_must_match: false, is_all_containers: true }),
			(false, OrchestrationType::Kubernetes) => IsExactMatch::No(IsTryGetMatch::No),
		};
		global_options.get_container_name(container_name, options)?
	};

	match global_options.get_orchestration_type() {
		OrchestrationType::DockerCompose => {
			if is_also_create {
				RunCommand::exec_with_args("docker", ["up", &container_name])
			} else {
				RunCommand::exec_with_args("docker", ["start", &container_name])
			}
		},
		OrchestrationType::Kubernetes => {
			let mut arguments = vec!["TODO", &container_name];
			global_options.add_name_space(&mut arguments);
			RunCommand::exec_with_args("oc", arguments)
		},
	}
}

pub fn start_all(global_options: &GlobalOrchOptions, _is_also_create: bool) -> Result<()> {
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
