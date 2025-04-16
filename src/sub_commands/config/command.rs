use anyhow::Result;
use clap::Args;
use log::info;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use strum::VariantNames;
use super::{structure::{Config, Orchestration}, file_handler::FileHandler};
use crate::{Run, GlobalOptions};

#[derive(Args, Debug)]
#[command(about = "Modify the default behavior of the script.")]
#[command(long_about = "Modify the default behavior of the script.
Will enter into interactive CLI that will allow you to edit your config file.")]
pub struct Command;

// Utils for Interactive Shell
impl Command {
	fn style_key(key: &str) -> String {
		format!("{}{}{}",
			"[".bright_red(),
			key.bold().white(),
			"]".bright_red(),
		)
	}

	fn print_keybinds() {
		info!("Keybinds:
{} Select
{} Select and continue
{} Continue without selecting",
			Self::style_key("space"), Self::style_key("enter"), Self::style_key("q")
		);
	}

	fn get_theme() -> ColorfulTheme {
		ColorfulTheme::default()
	}
}

struct Wizard;
impl Wizard {
	fn run(global_options: &GlobalOptions) -> Result<Option<Config>> {
		let GlobalOptions { is_verbose: _, version, config } = global_options;

		let theme = Command::get_theme();
		info!("Welcome to the setup wizard 🧙");
		Command::print_keybinds();

		let orchestration = Select::with_theme(&theme)
			.with_prompt("Pick orchestration")
			.default(config.orchestration.discriminant() as usize)
			.items(Orchestration::VARIANTS)
			.interact()?;
		let orchestration = u8::try_from(orchestration)
			.expect("Could fail only if Orchestration has more than u8::MAX variants");
		let orchestration = Orchestration::from_repr(orchestration)
			.expect("dialoguer::prompts::select::Select insures only valid discriminant will be received");

		let orchestration = match orchestration {
			Orchestration::DockerCompose => Orchestration::DockerCompose,
			Orchestration::Kubernetes { .. } => {
				let input = Input::with_theme(&theme)
					.with_prompt("Pick Name-Space");
				let input = if let Orchestration::Kubernetes { name_space } = &config.orchestration {
					input.with_initial_text(name_space)
				} else {
					input
				};
				let name_space: String = input.interact_text()?;
				Orchestration::Kubernetes { name_space }
			},
		};

		Ok(Some(Config {
			version: version.clone(), // Always upgrade to the current version
			orchestration
		}))
	}
}

impl Run for Command {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		let config = Wizard::run(global_options)?;

		match config {
			Some(config) => FileHandler::save(&config, global_options)?,
			None => info!("Aborted."),
		}

		Ok(())
	}
}