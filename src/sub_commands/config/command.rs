use std::fmt::Display;
use tap::prelude::*;
use anyhow::Result;
use clap::Args;
use log::{info, warn};
use time::PrimitiveDateTime;
use colored::Colorize;
use dialoguer::{theme::{ColorfulTheme, Theme}, Input, MultiSelect, Select, Confirm, FuzzySelect};
use lazy_static::lazy_static;
use hashbrown::{HashMap, hash_map::{Entry, OccupiedEntry}, HashSet};
use strum::{EnumString, FromRepr, VariantNames};
use super::{
	structure::{Config, OrchestrationType, Orchestration, Kubernetes, Alias, ContainerName},
	file_handler::FileHandler,
	prompts::{prompter::{Prompter, from_repr, Item, Items, Options, Selection, SelectionReturn}}
};
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

lazy_static! {
    static ref CONTAINER_ALIAS_DATE_FORMAT: Vec<time::format_description::BorrowedFormatItem<'static>> = {
        time::format_description::parse("[day]/[month]/[year repr:last_two] [hour]:[minute]").unwrap()
    };
}
type ContainerNameRaw = String;

#[derive(Debug, Hash, PartialEq, Eq)]
struct ContainerAlias {
	pub name: Alias,
	pub ttl: Option<PrimitiveDateTime>
}

impl Display for ContainerAlias {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.ttl {
            // Some(ttl) => write!(formatter, "{} (TTL: {})", self.name, ttl),
			Some(ttl) => write!(formatter, "{} (TTL: {})",
				self.name,
                ttl.format(&CONTAINER_ALIAS_DATE_FORMAT).unwrap()
            ),
            None => write!(formatter, "{}", self.name),
        }
	}
}

struct Wizard2<'cfg, Theme: dialoguer::theme::Theme> {
	prompter: Prompter<Theme>,
	global_options: &'cfg GlobalOptions,
	config: Config,
}

impl <'cfg, Theme: dialoguer::theme::Theme> Wizard2<'cfg, Theme> {
	pub fn new(global_options: &'cfg GlobalOptions, theme: Theme) -> Self {
		Self {
			prompter: Prompter::new(theme),
			global_options,
			config: Config {
				version: global_options.version.clone(),
				orchestration: global_options.config.orchestration.clone(),
			}
		}
	}
}

impl <Theme: dialoguer::theme::Theme> Wizard2<'_, Theme> {
	fn run(self) -> Result<Option<Config>> {
		let result = if self.menu_1(Selection::default())? {
			Some(self.config)
		} else {
			None
		};
		Ok(result)
	}
}

impl <Theme: dialoguer::theme::Theme> Wizard2<'_, Theme> {
	fn menu_1(&self, mut select: Selection) -> Result<bool> {
		#[allow(clippy::items_after_statements)]
		#[derive(EnumString, FromRepr, VariantNames)]
		#[repr(u8)]
		enum Prefix {
			#[strum(serialize = "Container/pods alias")]
			Containers,
			#[strum(serialize = "💾 Save and quit")]
			SaveAndQuit,
			#[strum(serialize = "🚫 Quit without saving")]
			Quit,
		}

		loop {			
			let options = vec![
				Options::Item(Item::String(format!("Orchestration: {:?}", self.global_options.config.orchestration.variant)), true),
				Options::Items(Items::StrRefs(Prefix::VARIANTS), None),
			];
			let Selection { vec_index, options_index } = self.prompter.fuzzy_select("Choose a setting to edit", &options)?;
			match vec_index {
				0 => self.menu_2(Selection { vec_index: 0, options_index: 0 })?,
				1 => match from_repr!(Prefix, options_index) {
					Prefix::Containers => self.menu_4(Selection { vec_index: 0, options_index: 0 })?,
					Prefix::SaveAndQuit => return Ok(true),
					Prefix::Quit => return Ok(false),
				},
				_ => unreachable!("options has only 2 elements")
			}
			// select = selected; // TODO: fix this and `options` above need to relay on `select`
		}
	}
}

impl <Theme: dialoguer::theme::Theme> Wizard2<'_, Theme> {
	fn menu_2(&self, mut select: Selection) -> Result<()> {
		todo!()
	}

	fn menu_4(&self, mut select: Selection) -> Result<()> {
		todo!()
	}
}

struct Wizard;
impl Wizard {
	fn run(global_options: &GlobalOptions) -> Result<Config> {
		info!("Welcome to the setup wizard 🧙");
		Command::print_keybinds();

		let theme = Command::get_theme();
		let orchestration_type = Self::pick_orch_type(global_options, &theme)?;
		let orchestration = Self::pick_orch(global_options, &theme, orchestration_type)?;

		Ok(Config {
			version: global_options.version.clone(), // Always upgrade to the current version
			orchestration
		})
	}

	// Shaked-TODO:
	// * A lot of duplicate code here, can be merged.
	// * Can unify the look/feel of the prompts, same emojis, use "Go Back" options everywhere.
	// * Split the alias modification from the orchestration, allow the user to pick if to modify it specifically.

	fn pick_orch_type(global_options: &GlobalOptions, theme: &dyn Theme) -> Result<OrchestrationType> {
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

	fn pick_orch(global_options: &GlobalOptions, theme: &dyn Theme, variant: OrchestrationType) -> Result<Orchestration> {
		let result = match variant {
			OrchestrationType::DockerCompose => Orchestration {
				variant: OrchestrationType::DockerCompose,
				kubernetes: global_options.config.orchestration.kubernetes.clone(),
				aliases: Self::pick_orch_aliases(global_options, theme)?,
			},
			OrchestrationType::Kubernetes => {
				Orchestration {
					variant: OrchestrationType::Kubernetes,
					kubernetes: Some(
						Kubernetes {
							name_space: Self::pick_kubernetes_name_space(global_options, theme)?,
						}
					),
					aliases: Self::pick_orch_aliases(global_options, theme)?,
				}
			},
		};
		Ok(result)
	}

	fn pick_orch_aliases(global_options: &GlobalOptions, theme: &dyn Theme) -> Result<HashMap<Alias, ContainerName>> {
		let is_editing = Confirm::with_theme(theme)
			.with_prompt("Would you like to edit the orchestration aliases?")
			.default(false)
			.show_default(true)
			.wait_for_newline(false)
			.interact()?;

		if !is_editing {
			return Ok(global_options.config.orchestration.aliases.clone());
		}

		let mut reversed_aliases = Self::get_reverse_orch_aliases(&global_options.config.orchestration.aliases);

		loop {
			#[derive(EnumString, FromRepr, VariantNames)]
			#[repr(u8)]
			enum Operations {
				#[strum(serialize = "Add/Modify/Remove specific container name")]
				Modify,
				#[strum(serialize = "❌ Remove multiple container names")]
				MultiRemove,
				#[strum(serialize = "Continue to other configurations")]
				Quit,
			}

			let operation_index = Select::with_theme(theme)
				.with_prompt("What operation would you like to do?")
				.default(Operations::Quit as usize)
				.items(Operations::VARIANTS)
				.report(false)
				.interact()?;
			let operation = from_repr!(Operations, operation_index);

			match operation {
				Operations::Modify => Self::select_orch_container_name(theme, &mut reversed_aliases)?,
				Operations::MultiRemove => Self::multi_remove_orch_container_names(theme, &mut reversed_aliases)?,
				Operations::Quit => return Ok(Self::get_restore_orch_aliases(reversed_aliases)),
			}
		}
	}

	fn select_orch_container_name(theme: &dyn Theme, reversed_aliases: &mut HashMap<ContainerNameRaw, HashSet<ContainerAlias>>) -> Result<()> {
		let mut options = vec![
			"🆕 Add new container name",
			"Go back",
		];
		reversed_aliases
			.keys()
			.map(|alias | -> &str { alias.as_ref() })
			.pipe(|iter| options.extend(iter));
		
		let selection = FuzzySelect::with_theme(theme)
			.with_prompt("Pick the container name to modify")
			.default(0)
			.items(&options)
			.report(false)
			.interact()?;

		match selection {
			0 => {
				let new_container_name = Input::with_theme(theme)
					.with_prompt("Pick new container name")
					.report(false)
					.interact_text()?;
				let _ = reversed_aliases.try_insert(new_container_name, HashSet::default());
			},
			1 => {
				return Ok(());
			}
			index => {
				// SAFETY:
				// The index that return from `FuzzySelect` should correspond
				// to the index at `options`.
				let container_name = String::from(*unsafe { options.get_unchecked(index) });
				let Entry::Occupied(mut entry) = reversed_aliases.entry(container_name) else {
					unreachable!("`container_name` must be in reversed_aliases");
				};
				let is_force_remove_container = Self::select_specific_orch_alias(theme, &mut entry)?;
				if is_force_remove_container || entry.get().is_empty() {
					entry.remove_entry();
				}
			},
		}
		Self::select_orch_container_name(theme, reversed_aliases)
	}

	/// # Returns
	/// If true, the user wants chose to delete this entry.
	fn select_specific_orch_alias<S>(theme: &dyn Theme, container: &mut OccupiedEntry<'_, ContainerNameRaw, HashSet<ContainerAlias>, S>) -> Result<bool> {
		const OPTIONS: [&str; 4] = [
			"🆕 Add new alias to this container name",
			"❌ Remove this container name and all of his aliases",
			"❌ Remove multiple aliases",
			"Go Back",
		];
		// let mut dyn_options = vec![];

		let dyn_options = container
			.get()
			.iter()
			.collect::<Vec<&ContainerAlias>>();

		let selection = FuzzySelect::with_theme(theme)
			.with_prompt(format!("Pick the alias to modify (for container name: {:?})", container.key()))
			.default(0)
			.items(&OPTIONS)
			.items(&dyn_options)
			.report(false)
			.interact()?;

		match selection {
			0 => {
				let new_alias = Input::with_theme(theme)
					.with_prompt("Pick new alias name")
					.report(false)
					.interact_text()?;
				container.get_mut().insert(ContainerAlias { name: new_alias, ttl: Some(time::macros::datetime!(1994-10-25 15:00)) }); // Shaked-TODO: now
				Self::select_specific_orch_alias(theme, container)
			},
			1 => {
				Ok(true)
			},
			2 => {
				todo!()
				// Self::multi_remove_orch_alias(theme, container)?;
				// Self::select_specific_orch_alias(theme, container)
			},
			3 => {
				Ok(false)
			},
			index => {
				todo!()
				// index = index - OPTIONS.len();
				// SAFETY:
				// The index that return from `FuzzySelect` should correspond
				// to the index at `dyn_options`.
				// let alias = String::from(*unsafe { dyn_options.get_unchecked(index) });
				// Self::modify_specific_orch_alias(theme, &alias, container.get_mut())?;
				// Self::select_specific_orch_alias(theme, container)
			},
		}
	}

	fn modify_specific_orch_alias(theme: &dyn Theme, alias: &str, aliases: &mut HashSet<String>) -> Result<()> {
		#[derive(EnumString, FromRepr, VariantNames)]
		#[repr(u8)]
		enum Operations {
			#[strum(serialize = "♻️ Rename")]
			Rename,
			#[strum(serialize = "❌ Remove")]
			Remove,
		}

		let operation_index = Select::with_theme(theme)
			.with_prompt(format!("What operation would you like to do? (Reminder: {} to quit)", Command::style_key("q")))
			.default(Operations::Rename as usize)
			.items(Operations::VARIANTS)
			.report(false)
			.interact_opt()?;
		let Some(operation_index) = operation_index else {
			return Ok(());
		};
		let operation_index = u8::try_from(operation_index)
			.expect("Could fail only if Operations has more than u8::MAX variants");
		let operation = Operations::from_repr(operation_index)
			.expect("dialoguer::prompts::select::Select insures only valid discriminant will be received");

		match operation {
			Operations::Rename => {
				let new_alias = Input::with_theme(theme)
					.with_prompt("Rename the alias")
					.with_initial_text(alias)
					.report(false)
					.interact_text()?;

				aliases.remove(alias);
				aliases.insert(new_alias);
			},
			Operations::Remove => {
				aliases.remove(alias);
			},
		}

		Ok(())
	}

	fn multi_remove_orch_alias<S>(theme: &dyn Theme, container: &mut OccupiedEntry<'_, String, HashSet<String>, S>) -> Result<()> {
		let aliases = container
			.get()
			.iter()
			.cloned()
			.collect::<Vec<String>>();

		let selections = MultiSelect::with_theme(theme)
			.with_prompt(format!("Select which aliases to remove (Reminder: {} to quit)", Command::style_key("q")))
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
			container.get_mut().remove(alias);
		}

		Ok(())
	}

	fn multi_remove_orch_container_names(theme: &dyn Theme, reversed_aliases: &mut HashMap<ContainerNameRaw, HashSet<ContainerAlias>>) -> Result<()> {
		let container_names = reversed_aliases
			.keys()
			.cloned()
			.collect::<Vec<String>>();
		
		let selections = MultiSelect::with_theme(theme)
			.with_prompt(format!("Select which container names to {} remove (Reminder: {} to quit)", "completely".italic(), Command::style_key("q")))
			.items(&container_names)
			.report(false)
			.interact_opt()?;
		let Some(selections) = selections else {
			return Ok(());
		};

		for index in selections {
			// SAFETY:
			// The indexes that return from `MultiSelect` should correspond
			// to the indexes at `container_names`.
			let container_name = unsafe { container_names.get_unchecked(index) };
			reversed_aliases.remove(container_name);
		}

		Ok(())
	}

	/// Generate a mapping from container names to their aliases,
	/// according to the current config.
	fn get_reverse_orch_aliases(aliases: &HashMap<Alias, ContainerName>) -> HashMap<ContainerNameRaw, HashSet<ContainerAlias>> {
		aliases
			.iter()
			.map(|(alias, container_name)| (alias.clone(), container_name.clone()))
			.fold(HashMap::new(), |mut result, (alias, ContainerName { name: container_name, ttl })| {
				result.entry(container_name).or_default().insert(ContainerAlias { name: alias, ttl });
				result
			})
	}

	/// Reverse a mapping from container names to their aliases,
	/// back to the config format.
	fn get_restore_orch_aliases(reversed_aliases: HashMap<ContainerNameRaw, HashSet<ContainerAlias>>) -> HashMap<Alias, ContainerName> {
		reversed_aliases
			.into_iter()
			.fold(HashMap::new(), |mut result, (container_name, aliases)| {
				for ContainerAlias { name: alias, ttl } in aliases {
					match result.entry(alias) {
						Entry::Vacant(entry) => {
							entry.insert(ContainerName { name: container_name.clone(), ttl });
						},
						Entry::Occupied(entry) => {
							warn!("The alias {0:?} points to two different container names: {1:?} and {2:?} ; Ignoring: {2:?}",
								entry.key(), entry.get(), ContainerName { name: container_name.clone(), ttl });
						},
					}
				}
				result
			})
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
		if let Some(config) = Wizard2::new(global_options, Self::get_theme()).run()? {
			FileHandler::save(&config, global_options)?;
		}
		Ok(())
	}
}
