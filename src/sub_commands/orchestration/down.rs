use anyhow::Result;
use clap::{Args, ArgAction::SetFalse};
use indexmap::indexset;
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch, MatchSource}};
use crate::{sub_commands::config::OrchestrationType, utils::{ExitError, RunCommand}};

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
* `docker-compose` - If true, will do the equivalent to `docker remove` instead of `docker kill`.
* `kubernetes` - If true, will reduce the deployment instead of of killing the pod (and the deployment might raise it back up).")]
// Shaked-TODO: might have better `long_help` once I know the commands.
	pub is_remove: bool,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		match self.container_name {
			Some(container_name) => stop(global_options, container_name, self.is_remove),
			None => stop_all(global_options, self.is_remove),
		}
	}
}

#[allow(clippy::items_after_statements, unreachable_code, unused)]
pub fn stop(global_options: &GlobalOrchOptions, container_name: String, is_remove: bool) -> Result<()> {
	ExitError::NotYetImplemented.exit("down::stop");
	const IS_EXACT_MATCH: bool = false; // Shaked-TODO: receive it as optional argument.
	let container_name = {
		let options = if IS_EXACT_MATCH {
			IsExactMatch::Yes
		} else {
			let match_source = match (global_options.get_orchestration_type(), is_remove) {
				(OrchestrationType::DockerCompose, _) => MatchSource::ExitingContainers{ is_all: true },
				(OrchestrationType::Kubernetes, false) => MatchSource::ExitingContainers{ is_all: false },
				(OrchestrationType::Kubernetes, true) => MatchSource::Config,
			};
			IsExactMatch::No(IsTryGetMatch::Yes(indexset! {match_source}))
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

#[allow(clippy::items_after_statements, unreachable_code, unused)]
pub fn stop_all(global_options: &GlobalOrchOptions, _is_remove: bool) -> Result<()> {
	ExitError::NotYetImplemented.exit("down::stop_all");
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
