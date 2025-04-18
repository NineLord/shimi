use anyhow::Result;
use clap::{Args, ArgAction::SetFalse};
use super::{command::RunOrchestration, global_orch_options::GlobalOrchOptions};
use crate::{utils::RunCommand, sub_commands::config::OrchestrationType};

/// Starts running container(s)
#[derive(Args, Debug)]
#[command(visible_alias = "start")]
pub struct Arguments {
	/// The name of the container that going to start running.
	/// If not given, will start all the containers.
	pub container_name: Option<String>,

	// Shaked-TODO: add to all docker commands, "exact match" for container_name, to indicate it shouldn't go throw the alias system.

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
	// Shaked-TODO: that's going to have some problems:
	// 1. it should go through the alias system, if the `s d ps` didn't help,
	// then it should return the alias system result instead of tell us it couldn't find the container.
	// (because the container could be down).
	// It should separate this case for docker-compose and kub, in kub there is no need to even check the `s d ps`.
	// 2. It should try and get all the containers (even those that are down).
	// There is no such case in kub state, do it only for docker-compose.
	let container_name = global_options.get_container_name(container_name)?;

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
