#![warn(clippy::pedantic, clippy::nursery, clippy::perf, clippy::correctness)]
#![allow(unused, dead_code)] // Shaked-TODO: delete this

pub mod parser;
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

use std::env;
use anyhow::Result;
use log::{Level, LevelFilter};
use colog::format::CologStyle;
use colored::Colorize;
// use time::{OffsetDateTime};
use clap::Parser;
use parser::TopCommand;

struct CustomLevelToken<'a, 'b> {
	time_format: &'b [time::format_description::BorrowedFormatItem<'a>],
}
impl CustomLevelToken<'_, '_> {
	pub const fn new() -> Self {
		Self {
			time_format: time::macros::format_description!("[day]/[month]/[year] [hour]:[minute]:[second].[subsecond digits:3]")
		}
	}
}
impl CologStyle for CustomLevelToken<'_, '_> {
    fn level_token(&self, level: &Level) -> &str {
        match *level {
            Level::Error => "Error",
            Level::Warn => "Warn",
            Level::Info => "Info",
            Level::Debug => "Debug",
            Level::Trace => "Trace",
        }
    }

	fn prefix_token(&self, level: &Level) -> String {
        format!(
			"{} {}{}{}",
			time::OffsetDateTime::now_local().unwrap().format(&self.time_format).unwrap(),
			"[".blue().bold(),
			self.level_color(level, self.level_token(level)),
			"]".blue().bold()
		)
    }
}

pub trait Run : Sized {
	/// # Errors
	/// Should return an error with explanation why the command couldn't run.
	fn run(self, global_options: GlobalOptions) -> Result<()>;
}

fn main() {
	let mut builder = env_logger::Builder::new();
	builder.format(colog::formatter(CustomLevelToken::new()));
	builder.filter(None, LevelFilter::Info);
    if let Ok(env) = env::var("SHIMI_LOG") {
        builder.parse_filters(&env);
    }
	builder.init();

	let mut commands = TopCommand::parse();
	if commands.is_verbose {
		println!("Input:\n{commands:#?}");
	}
	let (global_options, command) = commands.into_split();
	if let Err(error) = command.run(global_options) {
		eprintln!("{error}");
	}
}
