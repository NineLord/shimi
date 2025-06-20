mod common;

// TODO: use those to test the other commands like `s dx`.
// # assert_cmd = { version = "2.0.17" }
// # predicates = { version = "3.1.3" }
// use assert_cmd::Command;
// use predicates::str::contains;
// let mut cmd = Command::cargo_bin("s").unwrap();
// let assert = cmd
// 	.arg("config")
//     .assert();
// assert
//     .success()
//     .stdout(contains("fjdklafjd"));

#[cfg(feature = "dry_run")]
mod tests {
	use super::*;
	use common::*;
	use anyhow::Result;
	use rexpect::session::spawn_command;
	use assertables::assert_contains;
	use tap::Tap;

	#[test]
	fn save_config() -> Result<()> {
		remove_previous_config()?;
		let command = get_command()
			.tap_mut(|c| { c.arg("config"); });
		let output = {
			let mut process = spawn_command(command, Some(2_000))?;
			process.send(Keys::ArrowDown.into())?;
			process.send(Keys::ArrowDown.into())?;
			process.send(Keys::Enter.into())?;
			process.exp_eof()?
		};
		let output = clean_string(&output);
		let last_line = get_last_line(&output)?;
		assert_contains!(last_line, "Config file updated");
		Ok(())
	}
}
