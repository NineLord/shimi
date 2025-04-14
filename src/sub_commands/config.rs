use anyhow::Result;
use clap::Args;
use log::info;
use crate::{Run, GlobalOptions};

/// Modify the default behavior of the script.
#[derive(Args, Debug)]
pub struct Command;

impl Run for Command {
	fn run(self, _global_options: GlobalOptions) -> Result<()> {
		info!("Hello");
		Ok(())
	}
}