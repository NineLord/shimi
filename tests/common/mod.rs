#![allow(dead_code)]
use anyhow::{Result, anyhow};
use regex::Regex;
use lazy_static::lazy_static;
use const_format::concatcp;

lazy_static! {
    static ref REGEX_ASCII_ESCAPE_CODES: Regex = {
		Regex::new(r"\x1B\[[0-9;?]*[ -/]*[@-~]").unwrap()
    };
}

pub const ARROW_UP: &str = "\x1B[A";
pub const ARROW_DOWN: &str = "\x1B[B";
pub const ARROW_RIGHT: &str = "\x1B[C";
pub const ARROW_LEFT: &str = "\x1B[D";

pub const ENTER_C: char = '\n';
pub const ENTER: &str = concatcp!(ENTER_C);

pub fn clean_string(input: &str) -> String {
    REGEX_ASCII_ESCAPE_CODES.replace_all(input, "")
		.chars()
        .filter(|c| c.is_ascii())
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
		.filter(|c| *c != '\r')
        .collect()
}

pub fn get_last_line(input: &str) -> Result<&str> {
	input.split(ENTER_C)
		.map(|line| line.trim())
		.filter(|line| !line.is_empty())
		.last()
		.ok_or_else(|| anyhow!("There is no last line"))
}
