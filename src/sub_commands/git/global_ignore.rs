use std::{
	ffi::OsString,
	fs,
	io::Write,
	os::unix::process::CommandExt,
	path::{Path, PathBuf},
	process::Command
};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use log::debug;
use tap::Tap;
use crate::{prelude::*, utils::{default_value_home_dir, RunCommand}, GlobalOptions, Run};

use super::command;

const DEFAULT_FILE_NAME: &str = ".gitignore";
const COMMON_IGNORES: &[u8] = b"junk
target
node_modules
";
fn default_path() -> Result<OsString> {
	Ok(
		default_value_home_dir()?
			.into_os_string()
	)
}

#[derive(Args, Debug)]
#[command(about = "Sets and create a global '.gitignore' file.")]
#[command(long_about = "Sets and create a global '.gitignore' file.
Won't delete the previous file, just redirect to the new one.
If the file is empty will also append common ignores to it.")]
pub struct Arguments {
	/// The name of the '.gitignore' file.
	#[arg(short, long, default_value = DEFAULT_FILE_NAME)]
	pub file_name: String,

	/// Path to the '.gitignore' file.
	#[arg(short, long, default_value = default_path().unwrap())]
	pub path: PathBuf,
}

impl Run for Arguments {
	fn run(self, global_options: GlobalOptions) -> Result<()> {
		let Self { mut path, file_name } = self;
		
		if !cfg!(feature = "dry_run") {
			fs::create_dir_all(&path)
				.context("Failed to create the given path")?;
		}
		
		path.push(file_name);

		let mut git_ignore_file = fs::OpenOptions::new()
			.append(true)
			.create(true)
			.open(&path)?;

		if git_ignore_file.metadata()?.len() == 0 {
			if cfg!(feature = "dry_run") {
				debug!("The file is empty, would add common ignores to it.");
			} else {
				git_ignore_file.write_all(COMMON_IGNORES)?;
			}
		}

		RunCommand::run_with_args_sync("git", [
			"config", "--global", "core.excludesfile", path.to_string_lossy().as_ref()
			])?
			.exit_ok()?;
		
		Ok(())
	}
}