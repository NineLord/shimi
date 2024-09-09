use std::{os::unix::process::CommandExt, process::Command, error::Error};
use clap::Args;
use email_address::EmailAddress;
use crate::{Run, GlobalOptions};

use super::command;

/// Generate SSH key.
#[derive(Args, Debug)]
#[command(long_about = "Generate SSH key.
Useful for first time setup ssh connection with your git account.
Which will allow you to `git clone`/etc.")]
pub struct Arguments {
	/// The email of your git account.
	#[arg(value_parser = is_valid_email)]
	pub email: String
}

fn is_valid_email(input: &str) -> Result<String, String> {
	if EmailAddress::is_valid(input) {
		Ok(input.to_owned())
	} else {
		Err(format!("Invalid e-mail address"))
	}
}

impl Run for Arguments {
	#[inline]
	fn run(self, global_options: GlobalOptions) -> Result<(), Box<dyn Error>> {
		let mut command = Command::new("ssh-keygen");
		command
			.arg("-t").arg("ed25519")
			.arg("-C").arg(self.email);
		if global_options.is_debug {
			println!("Debug Mode: {command:?}");
			Ok(())
		} else {
			Err(Box::new(command.exec()))
		}
	}
}