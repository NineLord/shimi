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
	use rexpect::spawn;
	use assertables::assert_contains;
	use const_format::formatcp;

	#[test]
	fn save_config() -> Result<()> {
		let output = {
			let mut process = spawn(formatcp!("{:?} config", env!("CARGO_BIN_EXE_s")), Some(2_000))?;
			process.send(ARROW_DOWN)?;
			process.send(ARROW_DOWN)?;
			process.send(ENTER)?;
			process.exp_eof()?
		};
		let output = clean_string(&output);
		let last_line = get_last_line(&output)?;
		assert_contains!(last_line, "Config file updated");
		Ok(())
	}
}
