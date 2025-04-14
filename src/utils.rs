use std::{ffi::OsStr, os::unix::process::CommandExt, path::PathBuf, process::{Command, ExitStatus, Output}};
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
	/// # Errors
	/// Might fail to spawn the process.
	fn create_command<C, I, S>(command: C, arguments: Option<I>) -> Command
	where
		C: AsRef<OsStr>,
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		let mut command = Command::new(command);
		if let Some(arguments) = arguments {
			command.args(arguments);
		}

		debug!("Running Command: {command:?}");

		command
	}
}

// Impl run_sync
impl RunCommand {
	/// Runs a command and returns the output from it when it completely done.
	/// # Errors
	/// Might fail to spawn the process.
	pub fn run_sync<C: AsRef<OsStr>>(command: C) -> Result<Output> {
		Self::run_opt_args_sync(command, None as Option<&[&str]>)
	}

	/// Runs a command and returns the output from it when it completely done.
	/// # Errors
	/// Might fail to spawn the process.
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
	/// Might fail to spawn the process.
	fn run_opt_args_sync<C, I, S>(command: C, arguments: Option<I>) -> Result<Output>
	where
		C: AsRef<OsStr>,
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		let mut command = Self::create_command(command, arguments);

		if cfg!(feature = "dry_run") {
			Ok(Output { status: ExitStatus::default(), stdout: vec![], stderr: vec![] })
		} else {
			command
				.output()
				.context("Failed to run the given command")
		}
	}
}

// Impl exec
impl RunCommand {
	/// Runs a command and this process becomes the given command process.
	/// # Errors
	/// Might fail to spawn the process.
	pub fn exec<C: AsRef<OsStr>>(command: C) -> Result<()> {
		Self::exec_opt_args(command, None as Option<&[&str]>)
	}

	/// Runs a command and this process becomes the given command process.
	/// # Errors
	/// Might fail to spawn the process.
	pub fn exec_with_args<C, I, S>(command: C, arguments: I) -> Result<()>
	where
		C: AsRef<OsStr>,
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		Self::exec_opt_args(command, Some(arguments))
	}

	/// Runs a command and this process becomes the given command process.
	/// # Errors
	/// Might fail to spawn the process.
	fn exec_opt_args<C, I, S>(command: C, arguments: Option<I>) -> Result<()>
	where
		C: AsRef<OsStr>,
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		let mut command = Self::create_command(command, arguments);

		if cfg!(feature = "dry_run") {
			Ok(())
		} else {
			Err(anyhow!(command.exec())
				.context("Failed to run the given command"))
		}
	}
}