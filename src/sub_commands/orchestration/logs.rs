use anyhow::Result;
use clap::{Args, ArgAction::{SetFalse, SetTrue}};
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch}};
use crate::{utils::RunCommand, sub_commands::config::OrchestrationType, commands::GetSubCommandAliases};

const VISIBLE_ALIASES: [&str ; 2] = ["l", "log"];

/// Fetch the logs of a container
#[derive(Args, Debug)]
#[command(visible_aliases = VISIBLE_ALIASES)]
pub struct Arguments {
	/// The name of the container that going to show his logs
	pub container_name: String,

	/// Do not follow log output
	#[arg(long = "no-follow", action = SetFalse)]
	pub follow: bool,

	/// Number of lines to show from the end of the logs
	#[arg(short, long)]
	pub tail: Option<usize>,

	/// If true, won't try to convert the container name to his alias.
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
		let container_name = {
			let options = if self.is_exact_match {
				IsExactMatch::Yes
			} else {
				IsExactMatch::No(IsTryGetMatch::default())
			};
			global_options.get_container_name(self.container_name, options)?
		};

		let mut arguments = vec!["logs", &container_name];
		if self.follow {
			arguments.push("--follow");
		}

		let tail = self.tail
			.map(|num| num.to_string());
		if let Some(tail) = &tail {
			arguments.push("--tail");
			arguments.push(tail);
		}

		match global_options.get_orchestration_type() {
			OrchestrationType::DockerCompose => {
				RunCommand::exec_with_args("docker", arguments)
			},
			OrchestrationType::Kubernetes => {
				global_options.add_name_space(&mut arguments);
				RunCommand::exec_with_args("oc", arguments)
			},
		}
	}
}
