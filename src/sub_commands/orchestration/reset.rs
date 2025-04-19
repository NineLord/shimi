use anyhow::Result;
use clap::{Args, ArgAction::{SetFalse, SetTrue}};
use super::{command::RunOrchestration, global_orch_options::GlobalOrchOptions, up, down};

/// Restart container(s)
#[derive(Args, Debug)]
#[command(visible_alias = "restart")]
pub struct Arguments {
	/// The name of the container that going to restart running.
	/// If not given, will restart all the containers.
	pub container_name: Option<String>,

	#[arg(long = "no-remove", action = SetFalse,
	help = "Remove a container instead of stopping it",
	long_help = "Remove a container instead of stopping it during the restart.
Will be effective only when 'docker-compose' is the chosen orchestration.")]
	pub is_remove: bool,

	/// If true, won't try to convert the container name to his alias.
	/// Will be effective only when `CONTAINER_NAME` is given.
	// Shaked-TODO: maybe can ArgGroups with `container_name`?
	#[arg(short = 'e', long = "exact-match", action = SetTrue)]
	pub is_exact_match: bool,
}

impl RunOrchestration for Arguments {
	fn run_orch(self, global_options: &GlobalOrchOptions) -> Result<()> {
		if let Some(container_name) = self.container_name {
			down::stop(global_options, container_name.clone(), self.is_exact_match, self.is_remove)?;
			up::start(global_options, container_name, self.is_exact_match, self.is_remove)
		} else {
			down::stop_all(global_options, self.is_remove)?;
			up::start_all(global_options, self.is_remove)
		}
	}
}
