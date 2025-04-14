use anyhow::Result;
use clap::Args;
use crate::{utils::{is_valid_email, RunCommand}, GlobalOptions, Run, prelude::*};

#[derive(Args, Debug)]
#[command(about = "Set globally your user name and email.")]
#[command(long_about = "Set globally your user name and email.
Useful if this is main workstation, not shared with others.
Which will allow you to `git push`/etc.")]
pub struct Arguments {
	/// The user name of your git account.
	pub user_name: String,

	/// The email of your git account.
	#[arg(value_parser = is_valid_email)]
	pub email: String,
}

impl Run for Arguments {
	fn run(self, _global_options: GlobalOptions) -> Result<()> {
		RunCommand::run_with_args_sync("git", ["config", "--global", "user.name", &self.user_name])?
			.exit_ok()?;
		RunCommand::run_with_args_sync("git", ["config", "--global", "user.email", &self.email])?
			.exit_ok()?;
		Ok(())
	}
}