use std::{env, ffi::OsString, process, iter};
use anstyle::{AnsiColor, Color, Effects, RgbColor, Style};
use anyhow::Result;
use clap::{builder::Styles, ArgAction::SetTrue, ArgMatches, CommandFactory, FromArgMatches, Parser, Subcommand};
use log::{warn, error};
use strum::{VariantArray, IntoStaticStr, EnumDiscriminants};
use itertools::Itertools;
use lazy_static::lazy_static;
use hashbrown::HashMap;
use const_format::concatcp;
use simply_colored::{DIM_YELLOW, RESET, UNDERLINE};
use crate::{commands::{ExpandedAlias, GetAllExpandedAliases, GetExpandedAliases, GetSubCommandAliases, GetSubCommandsNames, Run}, logger, sub_commands::{config::{self, Config}, git, orchestration}};

#[allow(clippy::needless_raw_string_hashes)]
const SHIMI_ASCII: &str = r#"
   _____ __    _           _ 
  / ___// /_  (_)___ ___  (_)
  \__ \/ __ \/ / __ `__ \/ / 
 ___/ / / / / / / / / / / /  
/____/_/ /_/_/_/ /_/ /_/_/   
"#;
const SHIMI_COLORED: &str = concatcp!(DIM_YELLOW, SHIMI_ASCII, RESET);
const NOTE_COLORED: &str = concatcp!(UNDERLINE, "Note:", RESET);
const STYLES: Styles = Styles::styled() // Cargo colors: https://github.com/crate-ci/clap-cargo/blob/fa44ab6d7b756d69b5e2a92364f3a8a02a2fdab7/src/style.rs
		.usage(AnsiColor::BrightRed.on_default().bold().underline())
		.header(Style::new().fg_color(Some(Color::Rgb(RgbColor(255, 194, 102)))).bold().underline())
		.literal(AnsiColor::BrightWhite.on_default().bold())
		.placeholder(AnsiColor::BrightWhite.on_default().italic())
		.error(AnsiColor::BrightRed.on_default().effects(Effects::CURLY_UNDERLINE))
		.valid(AnsiColor::Green.on_default().bold())
		.invalid(AnsiColor::BrightRed.on_default());

#[derive(Parser, Debug)]
#[command(name = "s", bin_name = "s")]
#[command(about = "Common shortcuts for developers.")]
#[command(about = format!("{SHIMI_COLORED}
Common shortcuts for developers.
{NOTE_COLORED} Commands with sub-commands can be written without space between them."))]
#[command(long_about = format!("{SHIMI_COLORED}
A Script of common things a developer might need.
It contains commands that are too inconvenient to type every time,
or just hard to remember.

{NOTE_COLORED} Commands with sub-commands can be written without space between them.
For example: s docker logs ...
Is the same as: s dockerlogs ..."))]
#[command(styles=STYLES)]
#[command(version)]
pub struct TopCommand {
	#[arg(short = 'v', long = "verbose", global = true, action = SetTrue,
		help = "Prints extra information about what happens at run time",
		long_help = "Prints extra information about what happens at run time.
In addition it changes the logger to TRACE.
You are able to change the logger level to any level (error/info/warn/debug/trace)
using the environment variable `SHIMI_LOG`.
The environment variable has higher priority to this flag."
	)]
	pub is_verbose: bool,

	#[command(subcommand)]
	pub command: SubCommands,
}

#[derive(Debug)]
pub struct GlobalOptions {
	pub is_verbose: bool,
	pub version: String,
	pub config: Config,
	pub is_fail_to_parse_config: bool,
}

lazy_static! {
    pub static ref MERGED_COMMANDS_NAMES: HashMap<OsString, ExpandedAlias> = {
		let mut map = HashMap::new();
		TopCommand::get_all_expanded_aliases()
			.for_each(|expanded_alias| {
				let ExpandedAlias { command, sub_command } = &expanded_alias;
				map.insert(OsString::from(format!("{command}{sub_command}")), expanded_alias);
			});
		map
    };
}

impl TopCommand {
	/// # Panics
	/// If missing version at `Cargo.toml`.
	#[must_use]
	pub fn parse() -> (GlobalOptions, SubCommands) {
		let (version, top_command) = Self::parse_version();
		logger::init(top_command.is_verbose);
		let Some(version) = version else {
			error!("Shimi script missing current version number");
			process::exit(1);
		};

		let (config, is_fail_to_parse_config) = match config::FileHandler::read(top_command.is_verbose) {
			Ok(Some(config)) => (config, false),
			Err(_) => (Config::default(version.clone()), false),
			Ok(None) => {
				warn!("Failed to parse previous config file.
could it be from previous versions of the tool? (Current version: {version:?})
Continuing with default config."); // No backward support as of yet.
				(Config::default(version.clone()), true)
			},
		};
		(
			GlobalOptions {
				is_verbose: top_command.is_verbose,
				config,
				is_fail_to_parse_config,
				version,
			},
			top_command.command
		)
	}

	/// Modified version of [`clap_builder::derive::Parser::parse()`]
	/// that also returned the version of the command.
	fn parse_version() -> (Option<String>, Self) {
		let command = <Self as CommandFactory>::command();
		let version = command.get_version().map(String::from);
		let mut matches = Self::get_expanded_matches(command);
        let result = <Self as FromArgMatches>::from_arg_matches_mut(&mut matches)
            .map_err(|error| {
				let mut command = <Self as CommandFactory>::command();
				error.format(&mut command)
			});
        match result {
            Ok(command) => (version, command),
            Err(error) => error.exit(),
        }
	}

	fn get_expanded_matches(command: clap::Command) -> ArgMatches {
		let mut input = env::args_os();
		
		let Some(exe_path) = input.next() else {
			return command.get_matches();
		};
		let mut result = Vec::with_capacity(10);
		result.push(exe_path);

		while let Some(arg) = input.next() {
			if arg.to_string_lossy().starts_with('-') { // Can be optimized to use `os_str_bytes` for Unix specifically.
				// Ignore all the first flags
				result.push(arg);
			} else if let Some(ExpandedAlias { command: sub_command, sub_command: sub_sub_command }) = MERGED_COMMANDS_NAMES.get(arg.as_os_str()) {
				// Found the first none flag argument and it's an alias
				result.push(OsString::from(sub_command));
				result.push(OsString::from(sub_sub_command));
				result.extend(input);
				return command.get_matches_from(result);
			} else {
				// Found the first none flag argument and isn't an alias
				return command.get_matches();
			}
		}
		
		// Couldn't find any none flag argument
		command.get_matches()
	}
}

#[derive(Subcommand, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(IntoStaticStr, VariantArray))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum SubCommands {
	Git(git::Command),
	Orchestration(orchestration::Command),
	Config(config::Command),
}

impl GetExpandedAliases for SubCommandsDiscriminants {
	fn get_expanded_aliases(self) -> impl Iterator<Item = ExpandedAlias> {
		let command: &'static str = self.into();
		let command = iter::once(command);
		let (command_aliases, sub_commands_names): (_, Vec<&'static str>) = match self {
			Self::Git => (
				git::Command::get_sub_command_aliases(),
				git::Command::get_sub_commands_names().collect()
			),
			Self::Orchestration => (
				orchestration::Command::get_sub_command_aliases(),
				orchestration::Command::get_sub_commands_names().collect()
			),
			Self::Config => (
				config::Command::get_sub_command_aliases(),
				config::Command::get_sub_commands_names().collect()
			),
		};

		let command_names = command_aliases.iter()
			.copied()
			.chain(command);

		command_names
			.cartesian_product(sub_commands_names)
			.map(|(command, sub_command)| ExpandedAlias { command, sub_command })
	}
}

impl GetAllExpandedAliases for TopCommand {
	fn get_all_expanded_aliases() -> impl Iterator<Item = ExpandedAlias> {
		SubCommandsDiscriminants::VARIANTS
			.iter()
			.flat_map(|command| command.get_expanded_aliases())
	}
}

impl Run for SubCommands {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		match self {
			Self::Git(command) => command.run(global_options),
			Self::Orchestration(command) => command.run(global_options),
			Self::Config(command) => command.run(global_options),
		}
	}
}
