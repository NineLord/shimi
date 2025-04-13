use std::{ffi::{OsStr, OsString}, path::PathBuf, process::{Command, Output}};
use anyhow::{anyhow, Context, Result};
use log::debug;
use email_address::EmailAddress;

pub use xdg_home::home_dir;

/// # Errors
/// The current operations system don't have home directory defined.
#[inline]
pub fn default_value_home_dir() -> Result<PathBuf> {
	home_dir()
		.ok_or_else(|| anyhow!("Couldn't get the user's home directory"))
}

/// # Errors
/// The given string isn't valid email address.
pub fn is_valid_email(input: &str) -> Result<String> {
	if EmailAddress::is_valid(input) {
		Ok(input.to_owned())
	} else {
		Err(anyhow!("Invalid e-mail address"))
	}
}

pub struct RunCommand;

impl RunCommand {
	/// Runs a command and returns the output from it when it completely done.
	/// # Errors
	/// Might fail to run the command (spawn the process).
	pub fn run_sync<C: AsRef<OsStr>>(command: C) -> Result<Output> {
		Self::run_opt_args_sync(command, None as Option<&[&str]>)
	}

	/// Runs a command and returns the output from it when it completely done.
	/// # Errors
	/// Might fail to run the command (spawn the process).
	pub fn run_with_args_sync<C, I, S>(command: C, arguments: I) -> Result<Output>
	where
		C: AsRef<OsStr>,
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		Self::run_opt_args_sync(command, Some(arguments))
	}

	/// Runs a command and returns the output from it when it completely done.
	/// # Errors
	/// Might fail to run the command (spawn the process).
	fn run_opt_args_sync<C, I, S>(command: C, arguments: Option<I>) -> Result<Output>
	where
		C: AsRef<OsStr>,
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		let mut command = Command::new(command);
		if let Some(arguments) = arguments {
			command.args(arguments);
		}

		debug!("Running Command: {command:?}\nidk\nwhat is going ong"); // Shaked-TODO: receive global options via global variable and check for debug mode

		command
			.output()
			.context("Failed to run the given command")
	}
}