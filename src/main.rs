#![warn(clippy::pedantic, clippy::nursery, clippy::perf, clippy::correctness)]
// #![allow(unused, dead_code)]
#![deny(unused_must_use)]

pub mod top_command;
pub mod logger;
pub mod utils;
pub mod prelude;
pub mod sub_commands {
	pub mod git {
		mod command;
		pub use command::Command;
		pub(super) mod ssh_key_gen;
		pub(super) mod global_user_email;
		pub(super) mod global_ignore;
	}
	pub mod orchestration {
		mod command;
		pub use command::Command;
		pub(super) mod process_status;
	}
	pub mod config {
		pub use command::Command;
		pub use structure::{Config, OrchestrationType};
		pub(crate) use file_handler::FileHandler;

		mod command;
		mod file_handler;
		mod structure;
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
