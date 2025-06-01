#![warn(clippy::pedantic, clippy::nursery, clippy::perf, clippy::correctness)]
// #![allow(unused, dead_code)]
#![deny(unused_must_use)]

pub mod top_command;
pub mod logger;
pub mod utils;
pub mod prelude;
pub mod prompts {
	pub mod prompter;
	pub mod selection;
	pub mod multi_select;
	pub mod fuzzy_select;
	pub mod select;
	pub mod input;
	pub mod confirm;
}
pub mod sub_commands {
	pub mod git {
		mod command;
		pub use command::Command;
		
		mod ssh_key_gen;
		mod global_user_email;
		mod global_ignore;
	}
	pub mod orchestration {
		mod command;
		pub use command::Command;
		mod global_orch_options;
		
		mod process_status;
		mod execute;
		mod logs;
		mod ip;
		mod port_forward;
		mod up;
		mod down;
		mod reset;
	}
	pub mod config {
		mod command;
		pub use command::Command;

		mod file_handler;
		pub(crate) use file_handler::FileHandler;

		mod structure;
		pub use structure::{Config, OrchestrationType};

		mod edit_structure;
	}
}
pub use top_command::GlobalOptions;

use anyhow::Result;
use log::{error, trace};
use top_command::TopCommand;

pub trait Run : Sized {
	/// # Errors
	/// Should return an error with explanation why the command couldn't run.
	fn run(self, global_options: &GlobalOptions) -> Result<()>;
}

fn main() {
	let (global_options, command) = TopCommand::parse();

	trace!("Input - Command:\n{command:#?}");
	trace!("Input - Global Options:\n{global_options:#?}");
	
	if let Err(error) = command.run(&global_options) {
		if global_options.is_verbose {
			error!("{error:?}");
		} else {
			error!("{error}");
		}
	}
}
