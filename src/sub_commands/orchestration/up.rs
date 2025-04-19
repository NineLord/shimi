use anyhow::Result;
use clap::{Args, ArgAction::{SetFalse, SetTrue}};
use indexmap::indexset;
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch, MatchSource}};
use crate::{sub_commands::config::OrchestrationType, utils::{ExitError, RunCommand}};

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

	/// If true, won't try to convert the container name to his alias.
	/// Will be effective only when `CONTAINER_NAME` is given.
	// Shaked-TODO: maybe can ArgGroups with `container_name`?
	#[arg(short = 'e', long = "exact-match", action = SetTrue)]
	pub is_exact_match: bool,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		match self.container_name {
			Some(container_name) => start(global_options, container_name, self.is_exact_match, self.is_also_create),
			None => start_all(global_options, self.is_also_create),
		}
	}
}

#[allow(clippy::items_after_statements, unreachable_code, unused)]
pub fn start(global_options: &GlobalOrchOptions, container_name: String, is_exact_match: bool, is_also_create: bool) -> Result<()> {
	ExitError::NotYetImplemented.exit("up::start");
	let container_name = {
		let options = match (is_exact_match, global_options.get_orchestration_type()) {
			(true, _) => IsExactMatch::Yes,
			(false, OrchestrationType::DockerCompose) => IsExactMatch::No(IsTryGetMatch::Yes(
				indexset! {MatchSource::ExitingContainers { is_all: true }, MatchSource::Config}
			)),
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

#[allow(clippy::items_after_statements, unreachable_code, unused)]
pub fn start_all(global_options: &GlobalOrchOptions, _is_also_create: bool) -> Result<()> {
	ExitError::NotYetImplemented.exit("up::start_all");
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
