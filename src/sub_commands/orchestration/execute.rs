use std::ffi::OsStr;
use anyhow::Result;
use clap::{Args, ArgAction::SetTrue};
use super::{command::RunOrchestration, global_orch_options::{GlobalOrchOptions, IsExactMatch, IsTryGetMatch}};
use crate::{utils::RunCommand, sub_commands::config::OrchestrationType, commands::GetSubCommandAliases};

const DEFAULT_COMMAND: &str = "/bin/bash";
const VISIBLE_ALIASES: [&str ; 4] = ["x", "exec", "ent", "enter"];

/// Run a command in a running container
#[derive(Args, Debug)]
#[command(visible_aliases = VISIBLE_ALIASES)]
pub struct Arguments {
	/// The name of the container that going to execute the command
	pub container_name: String,

	/// If true, won't try to convert the container name to his alias.
	#[arg(short = 'e', long = "exact-match", action = SetTrue)]
	pub is_exact_match: bool,

	/// Overwrite the default command
	#[arg(short, long, num_args = 1.., default_value = DEFAULT_COMMAND)]
	pub command: Vec<String>,
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

		match global_options.get_orchestration_type() {
			OrchestrationType::DockerCompose => {
				let prompt = ["exec", "--interactive", "--tty", &container_name];

				let arguments = prompt.iter().map(OsStr::new)
					.chain(self.command.iter().map(OsStr::new))
					.collect::<Vec<&OsStr>>();

				RunCommand::exec_with_args("docker", arguments)
			},
			OrchestrationType::Kubernetes => {
				let mut arguments = vec!["exec", "--stdin", "--tty"];
				global_options.add_name_space(&mut arguments);
				arguments.push(&container_name);
				self.command.iter().for_each(|command| arguments.push(command));
				RunCommand::exec_with_args("oc", arguments)
			},
		}
	}
}
