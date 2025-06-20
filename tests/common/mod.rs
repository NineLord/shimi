#![allow(dead_code)]
use std::process::Command;
use anyhow::{Result, anyhow};
use regex::Regex;
use lazy_static::lazy_static;
use strum::IntoStaticStr;
use shimi::sub_commands::config::file_handler::FileHandler;
pub use shimi::sub_commands::config::file_handler::ReadResult;

lazy_static! {
    static ref REGEX_ASCII_ESCAPE_CODES: Regex = {
		Regex::new(r"\x1B\[[0-9;?]*[ -/]*[@-~]").unwrap()
    };
}

#[derive(IntoStaticStr)]
pub enum Keys {
	#[strum(serialize = "\x1B[A")]
	ArrowUp,
	#[strum(serialize = "\x1B[B")]
	ArrowDown,
	#[strum(serialize = "\x1B[C")]
	ArrowRight,
	#[strum(serialize = "\x1B[D")]
	ArrowLeft,
	#[strum(serialize = "\n")]
	Enter,
}

pub fn clean_string(input: &str) -> String {
    REGEX_ASCII_ESCAPE_CODES.replace_all(input, "")
		.chars()
        .filter(|c| c.is_ascii())
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
		.filter(|c| *c != '\r')
        .collect()
}

pub fn get_last_line(input: &str) -> Result<&str> {
	let enter: &str = Keys::Enter.into();
	input.split(enter)
		.map(|line| line.trim())
		.filter(|line| !line.is_empty())
		.last()
		.ok_or_else(|| anyhow!("There is no last line"))
}

pub fn remove_previous_config() -> Result<()> {
	FileHandler::delete()
}

pub fn read_current_config() -> ReadResult {
	FileHandler::read(false)
}

pub fn get_command() -> Command {
	Command::new(env!("CARGO_BIN_EXE_s"))
}
