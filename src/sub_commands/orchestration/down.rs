use anyhow::Result;
use clap::{Args, ArgAction::{SetFalse, SetTrue}};
use indexmap::indexset;
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch, MatchSource}};
use crate::{sub_commands::config::OrchestrationType, utils::{ExitError, RunCommand}, commands::GetSubCommandAliases};

const VISIBLE_ALIASES: [&str ; 3] = ["stop", "remove", "rm"];

/// Stop running container(s)
#[derive(Args, Debug)]
#[command(visible_aliases = VISIBLE_ALIASES)]
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

	/// If true, won't try to convert the container name to his alias.
	/// Will be effective only when `CONTAINER_NAME` is given.
	// Shaked-TODO: maybe can ArgGroups with `container_name`?
	#[arg(short = 'e', long = "exact-match", action = SetTrue)]
	pub is_exact_match: bool,
}

impl GetSubCommandAliases for Arguments {
	fn get_sub_command_aliases() -> &'static [&'static str] {
		&VISIBLE_ALIASES
	}
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		match self.container_name {
			Some(container_name) => stop(global_options, container_name, self.is_exact_match, self.is_remove),
			None => stop_all(global_options, self.is_remove),
		}
	}
}

#[allow(clippy::items_after_statements, unreachable_code, unused)]
pub fn stop(global_options: &GlobalOrchOptions, container_name: String, is_exact_match: bool, is_remove: bool) -> Result<()> {
	ExitError::NotYetImplemented.exit_with_message("down::stop");
	let container_name = {
		let options = if is_exact_match {
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
			let mut arguments = vec!["no idea", &container_name];
			global_options.add_name_space(&mut arguments);
			RunCommand::exec_with_args("oc", arguments)
		},
	}
}

#[allow(clippy::items_after_statements, unreachable_code, unused)]
pub fn stop_all(global_options: &GlobalOrchOptions, _is_remove: bool) -> Result<()> {
	ExitError::NotYetImplemented.exit_with_message("down::stop_all");
	match global_options.get_orchestration_type() {
		OrchestrationType::DockerCompose => {
			RunCommand::exec_with_args("docker", ["up"])
		},
		OrchestrationType::Kubernetes => {
			let mut arguments = vec!["no idea"];
			global_options.add_name_space(&mut arguments);
			RunCommand::exec_with_args("oc", arguments)
		},
	}
}
