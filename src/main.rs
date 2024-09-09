#![allow(unused, dead_code)] // Shaked-TODO: delete this

pub mod parser;
pub mod sub_commands {
	pub mod git {
		mod command;
		pub use command::Command;
		pub(super) mod ssh_key_gen;
	}
}
pub use parser::GlobalOptions;
pub trait Run {
	#[inline]
	fn run(self, global_options: GlobalOptions) -> Result<(), Box<dyn std::error::Error>>;
}

use std::error::Error;
use clap::Parser;
use parser::TopCommand;

fn main() -> Result<(), Box<dyn Error>> {
	let mut commands = TopCommand::parse();
	if commands.is_debug {
		println!("Input:\n{commands:#?}");
	}
	commands.run()
}
