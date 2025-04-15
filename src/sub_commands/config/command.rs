use anyhow::Result;
use clap::{Args, ArgAction::SetTrue};
use log::info;
use super::{structure::Config, file_handler::FileHandler};
use crate::{Run, GlobalOptions};

#[derive(Args, Debug)]
#[command(about = "Modify the default behavior of the script.")]
#[command(long_about = "Modify the default behavior of the script.
Will enter into interactive CLI that will allow you to edit your config file.")]
pub struct Command {
	#[arg(short = 'w', long = "wizard", action = SetTrue,
		help = "If turned on, will start a wizard that will go through all the configurations.",
		long_help = "If turned on, will start a wizard that will go through all the configurations,
and allow you to edit each one of them."
	)]
	pub is_wizard: bool,
}

// Wizard
impl Command {
	fn wizard(global_options: &GlobalOptions) -> Option<Config> {
		let GlobalOptions { is_verbose: _, config } = global_options;

		todo!()
	}
}

// Manual
impl Command {
	fn manual(global_options: &GlobalOptions) -> Option<Config> {
		unimplemented!()
	}
}

impl Run for Command {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		let config = if self.is_wizard {
			Self::wizard(global_options)
		} else {
			Self::manual(global_options)
		};

		match config {
			Some(config) => FileHandler::save(&config)?,
			None => info!("Aborted."),
		}

		Ok(())
	}
}