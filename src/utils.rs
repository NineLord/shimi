use std::{process, fmt::Display, ffi::OsStr, os::unix::process::CommandExt, path::PathBuf, process::{Command, ExitStatus, Output}};
use anyhow::{anyhow, Context, Result};
use log::{debug, error};
use email_address::EmailAddress;
use xdg_home::home_dir;

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

		let output = if cfg!(feature = "dry_run") {
			Output { status: ExitStatus::default(), stdout: vec![], stderr: vec![] }
		} else {
			command
				.output()
				.context("Failed to run the given command")?
		};

		if !output.status.success() {
			ExitError::BadExitCode.exit_with_message(format!("Failed to run the command: {command:?}
Returned with bad exit code: {output:?}"));
		}

		Ok(output)
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

pub enum ExitError {
	/// The user didn't input the correct values.
	BadArgument,
	/// When running a external command and it returned with bad exit code.
	BadExitCode,
	/// For work in progress sections.
	NotYetImplemented,
	/// If the process was interrupted (Ctrl-C).
	Interrupted,
	/// Other reason.
	Other,
}
impl ExitError {
	pub fn exit_with_message(self, message: impl Display) -> ! {
		self.exit_handler(Some(message))
	}

	pub fn exit(self) -> ! {
		self.exit_handler(None::<&'static str>)
	}

	fn exit_handler(self, message: Option<impl Display>) -> ! {
		match (&self, message) {
			(Self::NotYetImplemented, Some(message)) => error!("Not yet implemented: {message}"),
			(Self::NotYetImplemented, None) => error!("Not yet implemented"),
			(_, Some(message)) => error!("{message}"),
			(_, None) => (),
		}
		match self {
			Self::BadArgument => process::exit(2),
			Self::Other | Self::BadExitCode | Self::NotYetImplemented | Self::Interrupted => process::exit(1),
		}
	}
}
