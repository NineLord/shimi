use std::{ffi::OsString, path::PathBuf};

use email_address::EmailAddress;

pub use xdg_home::home_dir;

#[inline]
#[must_use]
pub fn default_value_home_dir() -> PathBuf {
	home_dir()
		.expect("Couldn't get the user's home directory")
}

pub fn is_valid_email(input: &str) -> Result<String, String> {
	if EmailAddress::is_valid(input) {
		Ok(input.to_owned())
	} else {
		Err(format!("Invalid e-mail address"))
	}
}