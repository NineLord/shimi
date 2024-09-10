use std::{os::unix::process::CommandExt, process::Command, error::Error};
use clap::Args;
use crate::{Run, GlobalOptions, utils::is_valid_email};

use super::command;

/// Set globally your user name and email.
#[derive(Args, Debug)]
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
	#[inline]
	fn try_run(self, global_options: GlobalOptions) -> Result<(), Box<dyn Error>> {
		let mut set_user_name = Command::new("git");
		set_user_name
			.arg("config")
			.arg("--global").arg("user.name").arg(self.user_name);
		let mut set_email = Command::new("git");
		set_email
			.arg("config")
			.arg("--global").arg("user.email").arg(self.email);

		if global_options.is_debug {
			println!("Debug Mode: {set_user_name:?}");
			println!("Debug Mode: {set_email:?}");
			return Ok(());
		}

		if let Err(error) = set_user_name.output() {
			eprintln!("The command fail: {set_user_name:?}");
			return Err(Box::new(error));
		}

		if let Err(error) = set_email.output() {
			eprintln!("The command fail: {set_email:?}");
			return Err(Box::new(error));
		}
		
		Ok(())
	}
}