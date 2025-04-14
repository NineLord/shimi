use std::process::Output;
use anyhow::{anyhow, Result};

pub trait ExitOk {
	/// # Returns
	/// `StdOut` of the command.
	/// # Errors
	/// * Exit code isn't 0.
	/// * `StdErr` isn't empty.
	fn exit_ok(self) -> Result<Vec<u8>>;
}

impl ExitOk for Output {
	fn exit_ok(self) -> Result<Vec<u8>> {
		let Self { status, stdout, stderr } = self;
		if !status.success() {
			return Err(anyhow!("Bad exit code ; Code: {:?} ; StdOut: {stdout:?} ; StdErr: {stderr:?}", status.code()));
		}
		if !stderr.is_empty() {
			return Err(anyhow!("StdErr isn't empty ; Code: {:?} ; StdOut: {stdout:?} ; StdErr: {stderr:?}", status.code()));
		}

		Ok(stdout)
	}
}