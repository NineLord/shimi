use std::{os::unix::process::CommandExt, process::Command, error::Error};
use clap::Args;
use crate::{Run, GlobalOptions, utils::is_valid_email};

use super::command;

/// Generate SSH key.
#[derive(Args, Debug)]
#[command(visible_aliases = ["keygen", "ssh-keygen"])]
#[command(long_about = "Generate SSH key.
Useful for first time setup ssh connection with your git account.
Which will allow you to `git clone`/etc.")]
pub struct Arguments {
	/// The email of your git account.
	#[arg(value_parser = is_valid_email)]
	pub email: String
}

impl Run for Arguments {
	#[inline]
	fn try_run(self, global_options: GlobalOptions) -> Result<(), Box<dyn Error>> {
		let mut command = Command::new("ssh-keygen");
		command
			.arg("-t").arg("ed25519")
			.arg("-C").arg(self.email);
		if global_options.is_debug {
			println!("Debug Mode: {command:?}");
			return Ok(());
		}
		
		let error: Result<(), Box<dyn Error>> = Err(Box::new(command.exec()));
		eprintln!("The command fail: {command:?}");
		error
	}
}