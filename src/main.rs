#![warn(clippy::pedantic, clippy::nursery, clippy::perf, clippy::correctness)]
#![allow(unused, dead_code)] // Shaked-TODO: delete this

pub mod parser;
pub mod logger;
pub mod utils;
pub mod sub_commands {
	pub mod git {
		mod command;
		pub use command::Command;
		pub(super) mod ssh_key_gen;
		pub(super) mod global_user_email;
		pub(super) mod global_ignore;
	}
}
pub use parser::GlobalOptions;

use anyhow::Result;
use log::{error, trace};
use clap::Parser;
use parser::TopCommand;

pub trait Run : Sized {
	/// # Errors
	/// Should return an error with explanation why the command couldn't run.
	fn run(self, global_options: GlobalOptions) -> Result<()>;
}

fn main() {
	let mut commands = TopCommand::parse();

	logger::init(commands.is_verbose);

	trace!("Input:\n{commands:#?}");

	let (global_options, command) = commands.into_split();
	
	if let Err(error) = command.run(global_options) {
		error!("{error}");
	}
}
