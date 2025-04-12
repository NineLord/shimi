use std::{
	ffi::OsString,
	fs,
	io::Write,
	os::unix::process::CommandExt,
	path::{Path, PathBuf},
	process::Command
};
use anyhow::{anyhow, Result};
use clap::Args;
use tap::Tap;
use crate::{Run, GlobalOptions, utils::default_value_home_dir};

use super::command;

const DEFAULT_FILE_NAME: &str = ".gitignore";

fn default_path() -> OsString {
	default_value_home_dir()
		.tap_mut(|path| path.push(DEFAULT_FILE_NAME))
		.into_os_string()
}

/// Sets and create a global '.gitignore' file.
#[derive(Args, Debug)]
#[command(long_about = "Sets and create a global '.gitignore' file.
Won't delete the previous file, just redirect to the new one.
Optionally, appends paths to the file.")]
pub struct Arguments {
	/// Path to the '.gitignore' file.
	#[arg(short, long, default_value = default_value_home_dir().into_os_string())]
	pub path: PathBuf,

	/// The name of the '.gitignore' file.
	#[arg(short, long, default_value = DEFAULT_FILE_NAME)]
	pub file_name: String,

	/// Paths to append to the file (won't overwrite anything in the file).
	pub directories: Option<Vec<PathBuf>>,
}

impl Run for Arguments {
	#[inline]
	fn try_run(self, global_options: GlobalOptions) -> Result<()> {
		let Self { mut path, file_name, directories } = self;
		fs::create_dir_all(&path)?;
		path.push(file_name);
		let path_clone = path.clone();

		let mut git_ignore_file = fs::OpenOptions::new()
			.write(true)
			.append(true)
			.create(true)
			.open(path)?;
		
		if let Some(directories) = directories {
			let mut data = directories
				.into_iter()
				.map(|mut dir| dir.into_os_string()
					.into_string()
					.expect("Received string with none Unicode characters")
				)
				.reduce(|mut accumulator, current| {
					accumulator.push('\n');
					accumulator.push_str(&current);
					accumulator
				})
				.expect("Can't be empty, since the original directories was wrapped in Option; clap wouldn't let it be Some if it was empty Vec");
			data.insert_str(0, "\n");
			// git_ignore_file.write(data.as_ref());
		}

		let mut set_excludes_file = Command::new("git");
		set_excludes_file
			.arg("config")
			.arg("--global").arg("core.excludesfile").arg(path_clone.as_os_str());
		

		if global_options.is_verbose {
			println!("Running Command: {set_excludes_file:?}");
		}

		
		set_excludes_file.output()?;
		if let Err(error) = set_excludes_file.output() {
			eprintln!("The command fail: {set_excludes_file:?}");
			return Err(anyhow!(error));
		}
		
		Ok(())
	}
}