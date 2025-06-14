use anyhow::Result;
use clap::Args;
use crate::{utils::{is_valid_email, RunCommand}, GlobalOptions, commands::{GetSubCommandAliases, Run}};

const VISIBLE_ALIASES: [&str ; 2] = ["keygen", "ssh-keygen"];

#[derive(Args, Debug)]
#[command(visible_aliases = VISIBLE_ALIASES)]
#[command(about = "Generate SSH key")]
#[command(long_about = "Generate SSH key.
Useful for first time setup ssh connection with your git account.
Which will allow you to `git clone`/etc.")]
pub struct Arguments {
	/// The email of your git account.
	#[arg(value_parser = is_valid_email)]
	pub email: String
}

impl GetSubCommandAliases for Arguments {
	fn get_sub_command_aliases() -> &'static [&'static str] {
		&VISIBLE_ALIASES
	}
}

impl Run for Arguments {
	fn run(self, _global_options: &GlobalOptions) -> Result<()> {
		RunCommand::exec_with_args("ssh-keygen", ["-t", "ed25519", "-C", &self.email])
	}
}
