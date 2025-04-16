use tap::prelude::*;
use anyhow::Result;
use clap::Args;
use log::info;
use colored::Colorize;
use dialoguer::{theme::{ColorfulTheme, Theme}, Input, MultiSelect, Select, Confirm, FuzzySelect};
use hashbrown::{HashMap, HashSet};
use strum::{EnumString, FromRepr, VariantNames};
use super::{structure::{Config, OrchestrationType, Orchestration, Kubernetes}, file_handler::FileHandler};
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
	fn run(global_options: &GlobalOptions) -> Result<Config> {
		info!("Welcome to the setup wizard 🧙");
		Command::print_keybinds();

		let theme = Command::get_theme();
		let orchestration_type = Self::pick_orchestration_type(global_options, &theme)?;
		let orchestration = Self::pick_orchestration(global_options, &theme, orchestration_type)?;

		Ok(Config {
			version: global_options.version.clone(), // Always upgrade to the current version
			orchestration
		})
	}

	fn pick_orchestration_type(global_options: &GlobalOptions, theme: &dyn Theme) -> Result<OrchestrationType> {
		let orchestration_index = Select::with_theme(theme)
			.with_prompt("Pick orchestration")
			.default(global_options.config.orchestration.variant as usize)
			.items(OrchestrationType::VARIANTS)
			.interact()?;
		let orchestration = u8::try_from(orchestration_index)
			.expect("Could fail only if Orchestration has more than u8::MAX variants");
		Ok(
			OrchestrationType::from_repr(orchestration)
				.expect("dialoguer::prompts::select::Select insures only valid discriminant will be received")
		)
	}

	fn pick_orchestration(global_options: &GlobalOptions, theme: &dyn Theme, variant: OrchestrationType) -> Result<Orchestration> {
		let result = match variant {
			OrchestrationType::DockerCompose => Orchestration {
				variant: OrchestrationType::DockerCompose,
				kubernetes: global_options.config.orchestration.kubernetes.clone(),
				aliases: Self::pick_orchestration_aliases(global_options, theme)?,
			},
			OrchestrationType::Kubernetes => {
				Orchestration {
					variant: OrchestrationType::Kubernetes,
					kubernetes: Some(
						Kubernetes {
							name_space: Self::pick_kubernetes_name_space(global_options, theme)?,
						}
					),
					aliases: Self::pick_orchestration_aliases(global_options, theme)?,
				}
			},
		};
		Ok(result)
	}

	fn pick_orchestration_aliases(global_options: &GlobalOptions, theme: &dyn Theme) -> Result<HashMap<String, String>> {
		let is_editing = Confirm::with_theme(theme)
			.with_prompt("Would you like to edit the orchestration aliases?")
			.default(false)
			.show_default(true)
			.wait_for_newline(false)
			.interact()?;

		if !is_editing {
			return Ok(global_options.config.orchestration.aliases.clone());
		}

		let mut reversed_aliases = Self::get_reverse_orch_aliases(global_options);

		loop {
			#[derive(EnumString, FromRepr, VariantNames)]
			#[repr(u8)]
			enum Operations {
				#[strum(serialize = "Add or modify existing aliases")]
				AddOrModify,
				#[strum(serialize = "Remove existing aliases")]
				Remove,
				#[strum(serialize = "Continue to other configurations")]
				Quit,
			}

			let operation_index = Select::with_theme(theme)
				.with_prompt("What operation would you like to do?")
				.default(Operations::Quit as usize)
				.items(Operations::VARIANTS)
				.report(false)
				.interact()?;
			let operation_index = u8::try_from(operation_index)
				.expect("Could fail only if Operations has more than u8::MAX variants");
			let operation = Operations::from_repr(operation_index)
				.expect("dialoguer::prompts::select::Select insures only valid discriminant will be received");

			match operation {
				Operations::AddOrModify => Self::add_orchestration_aliases(theme, &mut reversed_aliases)?,
				Operations::Remove => Self::remove_orchestration_aliases(theme, &mut reversed_aliases)?,
				Operations::Quit => return Ok(Self::get_restore_orch_aliases(reversed_aliases)),
			}
		}
	}

	fn add_orchestration_aliases(theme: &dyn Theme, reversed_aliases: &mut HashMap<String, HashSet<String>>) -> Result<()> {
		let mut options = vec![
			"🆕 Add new alias"
		];
		reversed_aliases
			.keys()
			.map(|alias | -> &str { alias.as_ref() })
			.pipe(|iter| options.extend(iter));
		
		let selection = FuzzySelect::with_theme(theme)
			.with_prompt(format!("Pick your alias to modify (Reminder: {} to quit)", Command::style_key("q")))
			.default(0)
			.items(&options)
			.report(false)
			.interact_opt()?;

		let Some(selection) = selection else {
			return Ok(());
		};

		match selection {
			0 => {
				let new_alias = Input::with_theme(theme)
					.with_prompt("Pick new alias name")
					.report(false)
					.interact_text()?;
				let _ = reversed_aliases.try_insert(new_alias, HashSet::default());
				Self::add_orchestration_aliases(theme, reversed_aliases)
			},
			index => todo!(),
		}
	}

	fn remove_orchestration_aliases(theme: &dyn Theme, reversed_aliases: &mut HashMap<String, HashSet<String>>) -> Result<()> {
		let aliases = reversed_aliases
			.keys()
			.cloned()
			.collect::<Vec<String>>();
		
		let selections = MultiSelect::with_theme(theme)
			.with_prompt(format!("Select which aliases to {} remove (Reminder: {} to quit)", "completely".bold(), Command::style_key("q")))
			.items(&aliases)
			.report(false)
			.interact_opt()?;
		let Some(selections) = selections else {
			return Ok(());
		};

		for index in selections {
			// SAFETY:
			// The indexes that return from `MultiSelect` should correspond
			// to the indexes at `aliases`.
			let alias = unsafe { aliases.get_unchecked(index) };
			reversed_aliases.remove(alias);
		}

		Ok(())
	}

	fn get_reverse_orch_aliases(global_options: &GlobalOptions) -> HashMap<String, HashSet<String>> {
		todo!()
	}

	fn get_restore_orch_aliases(reversed_aliases: HashMap<String, HashSet<String>>) -> HashMap<String, String> {
		todo!()
	}

	fn pick_kubernetes_name_space(global_options: &GlobalOptions, theme: &dyn Theme) -> Result<String> {
		let input = Input::with_theme(theme)
			.with_prompt("Pick Name-Space");

		let input = if let Some(Kubernetes { name_space }) = &global_options.config.orchestration.kubernetes {
			input.with_initial_text(name_space)
		} else {
			input
		};

		Ok(input.interact_text()?)
	}
}

impl Run for Command {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		let config = Wizard::run(global_options)?;
		FileHandler::save(&config, global_options)?;
		Ok(())
	}
}