use std::{fmt::Display, rc::Rc};
use humantime::{parse_duration, parse_rfc3339_weak};
use anyhow::Result;
use clap::Args;
use log::{info, warn};
use time::{OffsetDateTime, PrimitiveDateTime};
use colored::Colorize;
use dialoguer::theme::{ColorfulTheme, Theme};
use lazy_static::lazy_static;
use hashbrown::{HashMap, hash_map::Entry, HashSet};
use indexmap::{map::Entry as IndexEntry, set::MutableValues, IndexMap, IndexSet};
use strum::{EnumString, FromRepr, VariantNames};
use super::{
	structure::{Config, OrchestrationType, Orchestration, Kubernetes, StrAlias, AliasToContainer, ContainerName},
	file_handler::FileHandler,
	prompts::{prompter::Prompter, selection::{Options, Selection, SelectionReturn, from_repr}, multi_select::ToDefaults, input::Input}
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

	static ref USER_PROMPT_DATE_FORMAT: Vec<time::format_description::BorrowedFormatItem<'static>> = {
        time::format_description::parse("[year repr:full]-[month]-[day] [hour]:[minute]:[second padding:zero]").unwrap()
    };
}

type RcContainerName = Rc<str>;
type RcAlias = Rc<str>;

#[derive(Debug, Hash, PartialEq, Eq)]
struct ContainerAlias {
	pub name: RcAlias,
	pub ttl: Option<PrimitiveDateTime>
}
impl Display for ContainerAlias {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.ttl {
			Some(ttl) => write!(formatter, "{} (TTL: {})",
				self.name,
                ttl.format(&CONTAINER_ALIAS_DATE_FORMAT).unwrap()
            ),
            None => write!(formatter, "{}", self.name),
        }
	}
}

type ContainerToAlias = IndexMap<RcContainerName, IndexSet<ContainerAlias>>;
type Aliases = HashSet<Rc<str>>;
#[derive(Debug)]
struct ContainerMapping {
	containers: ContainerToAlias,
	aliases: Aliases,
}
impl ContainerMapping {
	/// # Panics
	/// If the given `container_name` doesn't exists in the container mapping.
	fn get_display_aliases(&self, container_name: &str) -> Vec<String> {
		self.containers.get(container_name)
			.expect("The given container_name must be already in the container mapping")
			.iter()
			.map(std::string::ToString::to_string)
			.collect()
	}
}

struct Wizard<'cfg, T: Theme> {
	prompter: Prompter<T>,
	global_options: &'cfg GlobalOptions,
	config: Config,
}

impl <'cfg, T: Theme> Wizard<'cfg, T> {
	pub fn new(global_options: &'cfg GlobalOptions, theme: T) -> Self {
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

impl <T: Theme> Wizard<'_, T> {
	fn run(mut self) -> Result<Option<Config>> {
		let result = if self.menu_1_select_category(Selection::default())? {
			Some(self.config)
		} else {
			None
		};
		Ok(result)
	}
}

// TODO: probably didn't update the aliases along with the container names
impl <T: Theme> Wizard<'_, T> {
	fn menu_1_select_category(&mut self, mut select: Selection) -> Result<bool> {
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
			let options = Options::with_capacity(1 + Prefix::VARIANTS.len())
				.insert_string(format!("Orchestration: {:?}", self.config.orchestration.variant))
				.insert_str_refs(Prefix::VARIANTS)
				.set_selection(&select);
			let selected = self.prompter.fuzzy_select("Choose a setting to edit", &options)?;
			match selected.vec_index {
				0 => self.menu_2_set_orch(&SelectionReturn::default())?,
				1 => match from_repr!(Prefix, selected.options_index) {
					Prefix::Containers => self.menu_4_edit_containers(SelectionReturn::default())?,
					Prefix::SaveAndQuit => return Ok(true),
					Prefix::Quit => return Ok(false),
				},
				_ => unreachable!("options has only 2 elements")
			}
			select = selected;
		}
	}

	fn menu_2_set_orch(&mut self, select: &SelectionReturn) -> Result<()> {
		macro_rules! add_check_mark {
			($prefix:literal, $variant:expr, $expected:pat $(if $guard:expr)? $(,)?) => ({
				let check_mark = if matches!($variant, $expected) {
					" ✅"
				} else {
					""
				};
				format!("{}{check_mark}", $prefix)
			});
		}

		let mut options = Options::with_capacity(3)
			.insert_string(add_check_mark!("🐋 Docker-Compose", self.config.orchestration.variant, OrchestrationType::DockerCompose))
			.insert_string(add_check_mark!("☸️  Kubernetes", self.config.orchestration.variant, OrchestrationType::Kubernetes))
			.insert_return()
			.set_selection(select);

		loop {
			let selected = self.prompter.select_with_return("Choose orchestration", &options)?;
			match selected {
				SelectionReturn::Selection(Selection { vec_index: 0, options_index: _ }) => {
					self.config.orchestration.variant = OrchestrationType::DockerCompose;
					break;
				},
				SelectionReturn::Selection(Selection { vec_index: 1, options_index: _ }) => {
					if self.menu_3_set_kub_name_space()? {
						self.config.orchestration.variant = OrchestrationType::Kubernetes;
						break;
					}
				},
				SelectionReturn::Return => break,
				SelectionReturn::Selection(Selection { vec_index: _, options_index: _ }) => unreachable!("There are only two options currently"),
			}
			options = options.re_set_selection(&selected);
		}

		Ok(())
	}

	fn menu_3_set_kub_name_space(&mut self) -> Result<bool> {
		let previous_name_space: Option<&str> = match &self.config.orchestration.kubernetes {
			Some(Kubernetes { name_space }) => Some(name_space.as_ref()),
			None => None,
		};
		let input = self.prompter.input(
			"Choose name space (leave empty to not select Kubernetes)",
			previous_name_space
		)?;
		let result = match input {
			Input::NoneEmpty(name_space) => {
				self.config.orchestration.kubernetes = Some(Kubernetes { name_space });
				true
			},
			Input::Empty => false,
		};
		Ok(result)
	}

	fn menu_4_edit_containers(&mut self, mut select: SelectionReturn) -> Result<()> {
		#[derive(EnumString, FromRepr, VariantNames)]
		#[repr(u8)]
		enum Prefix {
			#[strum(serialize = "➕ Add")]
			Add,
			#[strum(serialize = "❌ Remove")]
			Remove,
		}

		let mut container_to_alias = self.get_reverse_orch_aliases();
		loop {
			let containers = container_to_alias.containers.keys().map(Rc::clone).collect::<Vec<Rc<str>>>();
			let options = Options::with_capacity(Prefix::VARIANTS.len() + containers.len() + 1)
					.insert_str_refs(Prefix::VARIANTS)
					.insert_rc_strings(&containers)
					.insert_return()
					.set_selection(&select);

			let mut selected = self.prompter.fuzzy_select_with_return("Choose container name", &options)?;
			match selected {
				SelectionReturn::Selection(Selection { vec_index: 0, options_index }) => {
					match from_repr!(Prefix, options_index) {
						Prefix::Add => { self.menu_5_new_container(&mut container_to_alias)?; },
						Prefix::Remove => self.menu_13_multi_remove_containers(&mut container_to_alias)?,
					}
				},
				SelectionReturn::Selection(Selection { vec_index: 1, options_index }) => {
					let container = containers.get(options_index)
						.expect("options_index has to be in range of containers");
					let is_removed = self.menu_7_edit_container(&mut container_to_alias, Rc::clone(container), SelectionReturn::default())?;
					if is_removed && containers.len() == 1 { // It was the last container
						selected = SelectionReturn::Return;
					}
				},
				SelectionReturn::Return => {
					self.set_restore_orch_aliases(container_to_alias.containers);
					return Ok(())
				},
				SelectionReturn::Selection(Selection { vec_index: _, options_index: _ }) => unreachable!("options has only 2 elements"),
			}
			select = selected;
		}
	}

	/// # Returns
	/// If `true`, added a container.
	fn menu_5_new_container(&self, container_to_alias: &mut ContainerMapping) -> Result<bool> {
		let input = self.prompter.input_with_validation("Choose new container name (leave empty to not add)", None,
		|container_name: &String | -> Result<(), String> {
			let container_name = container_name.trim();
			if container_name.is_empty() {
				Ok(())
			} else if container_to_alias.containers.contains_key(container_name) {
				Err(format!("The container name {container_name:?} already exists"))
			} else {
				Ok(())
			}
		})?;

		let result = match input {
			Input::NoneEmpty(container_name) => {
				let container_name = Rc::from(container_name);
				self.menu_6_new_alias(Rc::clone(&container_name), container_to_alias)?;
				!self.menu_7_edit_container(container_to_alias, container_name, SelectionReturn::default())?
			},
			Input::Empty => false,
		};

		Ok(result)
	}

	fn menu_6_new_alias(&self, container_name: Rc<str>, container_to_alias: &mut ContainerMapping) -> Result<()> {
		let input = self.prompter.input_with_validation(format!("Choose new alias for {container_name:?} (leave empty to not add)"), None,
		|alias: &String | -> Result<(), String> {
			let alias = alias.trim();
			if alias.is_empty() {
				Ok(())
			} else if container_to_alias.aliases.contains(alias) {
				Err(format!("The alias {alias:?} already exists"))
			} else {
				Ok(())
			}
		})?;

		let alias = match input {
			Input::NoneEmpty(alias) => Rc::from(alias),
			Input::Empty => return Ok(()),
		};

		let aliases = container_to_alias.containers.entry(container_name).or_default();

		let is_new_alias = container_to_alias.aliases.insert(Rc::clone(&alias));
		debug_assert!(is_new_alias, "The validation above checked that this is a new unique alias");
		let is_new_alias = aliases.insert(ContainerAlias { name: alias, ttl: None });
		debug_assert!(is_new_alias, "The validation above checked that this is a new unique alias");

		Ok(())
	}

	/// # Returns
	/// If `true`, the container was removed
	fn menu_7_edit_container(&self, container_to_alias: &mut ContainerMapping, mut container_name: Rc<str>, mut select: SelectionReturn) -> Result<bool> {
		#[derive(EnumString, FromRepr, VariantNames)]
		#[repr(u8)]
		enum EditAlias {
			#[strum(serialize = "➕ Add alias")]
			Add,
			#[strum(serialize = "❌ Remove alias")]
			Remove,
		}

		loop {
			let edit_container = [
				format!("✏️  Rename {container_name:?}"),
				format!("❌ Remove {container_name:?}"),
			];
			let aliases = container_to_alias.get_display_aliases(&container_name);
			let options = Options::with_capacity(EditAlias::VARIANTS.len() + aliases.len() + 1)
					.insert_str_refs(EditAlias::VARIANTS)
					.insert_strings(&edit_container)
					.insert_strings(&aliases)
					.insert_return()
					.set_selection(&select);

			let mut selected = self.prompter.fuzzy_select_with_return(format!("Editing {container_name:?}"), &options)?;
			match selected {
				SelectionReturn::Selection(Selection { vec_index: 0, options_index }) => {
					match from_repr!(EditAlias, options_index) {
						EditAlias::Add => self.menu_6_new_alias(Rc::clone(&container_name), container_to_alias)?,
						EditAlias::Remove => self.menu_8_multi_remove_aliases(&container_name, container_to_alias)?,
					}
				},
				SelectionReturn::Selection(Selection { vec_index: 1, options_index: 0 }) => {
					if let Some(new_container_name) = self.menu_11_rename_container(&container_name, &mut container_to_alias.containers)? {
						container_name = new_container_name;
					}
				},
				SelectionReturn::Selection(Selection { vec_index: 1, options_index: 1 }) => {
					if self.menu_12_confirm_remove_container(&container_name)? {
						return Ok(true);
					}
				},
				SelectionReturn::Selection(Selection { vec_index: 2, options_index }) => {
					let container_aliases = container_to_alias.containers.get_mut(&container_name)
						.expect("container_name must be inside container mapping");
					let alias = container_aliases
						.get_index_mut2(options_index)
						.expect("selection must be inside aliases ; aliases must be same order as container mapping");
					let is_removed = self.menu_9_edit_aliases(&container_name, alias, &mut container_to_alias.aliases, SelectionReturn::default())?;
					if is_removed {
						let removed_alias = container_aliases.shift_remove_index(options_index);
						if let Some(removed_alias) = removed_alias {
							let is_removed = container_to_alias.aliases.remove(&removed_alias.name);
							debug_assert!(is_removed, "an alias is suppose to be removed from aliases");
						} else {
							unreachable!("An alias is suppose to be removed from container");
						}
						if aliases.len() == 1 { // It was the last alias
							selected = SelectionReturn::Return;
						}
					}
				},
				SelectionReturn::Return => return Ok(false),
				SelectionReturn::Selection(Selection { vec_index: 1, options_index: _ }) => unreachable!("edit_container has only 2 elements"),
				SelectionReturn::Selection(Selection { vec_index: _, options_index: _ }) => unreachable!("options has only 3 elements"),
			}
			select = selected;
		}
	}

	fn menu_8_multi_remove_aliases(&self, container_name: &str, container_to_alias: &mut ContainerMapping) -> Result<()> {
		let aliases = container_to_alias.get_display_aliases(container_name);
		let selected = self.prompter.multi_select(
			format!("Remove aliases for {container_name:?} (q to return to previous menu without selecting)"),
			&aliases, None)?;

		let Some(selected) = selected else {
			return Ok(());
		};

		let aliases = container_to_alias.containers.get_mut(container_name)
			.expect("The given container_name must be already in the container mapping");
		for selected in selected.into_iter().rev() {
			let removed_alias = aliases.shift_remove_index(selected);
			debug_assert!(removed_alias.is_some(), "The index must be pointing to a valid alias");
		}
		
		Ok(())
	}

	/// # Returns
	/// If `true`, the alias needs to be removed.
	fn menu_9_edit_aliases(&self, container_name: &str, alias: &mut ContainerAlias, aliases: &mut Aliases, mut select: SelectionReturn) -> Result<bool> {
		#[derive(EnumString, FromRepr, VariantNames)]
		#[repr(u8)]
		enum EditAlias {
			#[strum(serialize = "✏️  Rename alias")]
			Rename,
			#[strum(serialize = "❌ Remove alias")]
			Remove,
		}

		loop {
			let edit_ttl = if alias.ttl.is_some() {
				vec!["✏️  Edit TTL", "❌ Remove TTL"]
			} else {
				vec!["➕ Add TTL"]
			};
			let options = Options::with_capacity(EditAlias::VARIANTS.len() + edit_ttl.len() + 1)
					.insert_str_refs(EditAlias::VARIANTS)
					.insert_str_refs(&edit_ttl)
					.insert_return()
					.set_selection(&select);

			let mut selected = self.prompter.select_with_return(format!("Editing {:?} for {container_name:?}", alias.name), &options)?;
			match (&mut alias.ttl, selected) {
				(_, SelectionReturn::Selection(Selection { vec_index: 0, options_index })) => {
					match from_repr!(EditAlias, options_index) {
						EditAlias::Rename => self.menu_10_rename_alias(container_name, alias, aliases)?,
						EditAlias::Remove => return Ok(true),
					}
				},
				(ttl, SelectionReturn::Selection(Selection { vec_index: 1, options_index: 0 })) => self.menu_15_edit_ttl(container_name, &alias.name, ttl)?,
				(Some(_), SelectionReturn::Selection(Selection { vec_index: 1, options_index: 1 })) => {
					alias.ttl = None;
					selected = SelectionReturn::Selection(Selection { vec_index: 1, options_index: 0 });
				},
				(_, SelectionReturn::Return) => return Ok(false),
				(Some(_), SelectionReturn::Selection(Selection { vec_index: 1, options_index: _ })) => unreachable!("edit_ttl with ttl has only 2 options"),
				(None, SelectionReturn::Selection(Selection { vec_index: 1, options_index: _ })) => unreachable!("edit_ttl without ttl has only 1 options"),
				(_, SelectionReturn::Selection(Selection { vec_index: _, options_index: _ })) => unreachable!("options has only 3 elements"),
			}
			select = selected;
		}
	}

	fn menu_10_rename_alias(&self, container_name: &str, alias: &mut ContainerAlias, aliases: &mut Aliases) -> Result<()> {
		let mut is_same_alias_name = false;
		let input = self.prompter.input_with_validation(
			format!("Choose new alias for {:?} of {container_name:?} (leave empty to not rename)", alias.name),
			Some(&alias.name),
			|new_alias: &String | -> Result<(), String> {
				let new_alias = new_alias.trim();
				if new_alias.is_empty() {
					Ok(())
				} else if alias.name.as_ref() == new_alias {
					is_same_alias_name = true;
					Ok(())
				} else if aliases.contains(new_alias) {
					Err(format!("The alias {new_alias:?} already exists"))
				} else {
					Ok(())
				}
			})?;

		if is_same_alias_name {
			return Ok(());
		}

		let new_alias = match input {
			Input::NoneEmpty(alias) => Rc::from(alias),
			Input::Empty => return Ok(()),
		};

		aliases.remove(&alias.name);
		alias.name = new_alias;
		aliases.insert(Rc::clone(&alias.name));

		Ok(())
	}

	fn menu_11_rename_container(&self, container_name: &str, container_to_alias: &mut ContainerToAlias) -> Result<Option<RcContainerName>> {
		let mut is_same_container_name = false;
		let input = self.prompter.input_with_validation(
			format!("Choose new container name for {container_name:?} (leave empty to not rename)"),
			Some(container_name),
			|new_container_name: &String | -> Result<(), String> {
				let new_container_name = new_container_name.trim();
				if new_container_name.is_empty() {
					Ok(())
				} else if container_name == new_container_name {
					is_same_container_name = true;
					Ok(())
				} else if container_to_alias.contains_key(new_container_name) {
					Err(format!("The container name {new_container_name:?} already exists"))
				} else {
					Ok(())
				}
			})?;
		
		if is_same_container_name {
			return Ok(None);
		}

		let new_container_name = match input {
			Input::NoneEmpty(container_name) => Rc::from(container_name),
			Input::Empty => return Ok(None),
		};

		let new_container_name_entry = container_to_alias.insert(Rc::clone(&new_container_name), IndexSet::new());
		debug_assert!(new_container_name_entry.is_none(), "There shouldn't be an entry for the new container name, it was validated in the input prompt");
		match container_to_alias.swap_remove(container_name) {
			Some(prev_aliases) => {
				let aliases = container_to_alias.get_mut(&new_container_name)
					.expect("The new container name was just inserted");
				*aliases = prev_aliases;
			},
			None => unreachable!("The current container name must have an entry in the mapping"),
		}

		Ok(Some(new_container_name))
	}

	fn menu_12_confirm_remove_container(&self, container_name: &str) -> Result<bool> {
		self.prompter.confirm(format!("Are you sure you want to remove {container_name:?}?"))
	}

	fn menu_13_multi_remove_containers(&self, container_to_alias: &mut ContainerMapping) -> Result<()> {
		let mut defaults = None;
		
		let selected = loop {
			let containers = container_to_alias.containers.keys()
				.map(std::convert::AsRef::as_ref)
				.collect::<Vec<&str>>();
			let selected = self.prompter.multi_select(
				"Remove multiple container names (q to return to previous menu without selecting)",
				&containers, defaults)?;

			let Some(selected) = selected else {
				return Ok(());
			};


			if self.menu_14_confirm_remove_containers()? {
				break selected;
			}

			defaults = Some(selected.to_defaults(containers.len()));
		};

		for selected in selected.into_iter().rev() {
			let removed_container = container_to_alias.containers.shift_remove_index(selected);
			debug_assert!(removed_container.is_some(), "The index must be pointing to a valid container");
		}
		
		Ok(())
	}

	fn menu_14_confirm_remove_containers(&self) -> Result<bool> {
		self.prompter.confirm("Are you sure you want to remove those containers?")
	}

	fn menu_15_edit_ttl(&self, container_name: &str, alias: &str, ttl: &mut Option<PrimitiveDateTime>) -> Result<()> {
		const HELP_SYNTAX: &str = "
Syntax for Durations:
* `nsec`, `ns` -- nanoseconds
* `usec`, `us` -- microseconds
* `msec`, `ms` -- milliseconds
* `seconds`, `second`, `sec`, `s`
* `minutes`, `minute`, `min`, `m`
* `hours`, `hour`, `hr`, `h`
* `days`, `day`, `d`
* `weeks`, `week`, `w`
* `months`, `month`, `M` -- defined as 30.44 days
* `years`, `year`, `y` -- defined as 365.25 days
Example:
`2h 37min`
`32ms`

Syntax for Timestamp (RFC3339-like):
1. Any precision of fractional digits `2018-02-14 00:28:07.133`.
2. Supports timestamp with or without either of `T` or `Z`.
3. Anything valid for [`parse_rfc3339`](parse_rfc3339) is valid.
4. Only UTC is supported, even if `Z` is not specified.
Example:
`1994-10-25 16:45:00`";

		let prompt = if ttl.is_some() {
			format!("Editing TTL for {alias:?} of {container_name:?} (type 'help' for syntax, leave empty to not change the TTL)")
		} else {
			format!("Adding TTL for {alias:?} of {container_name:?} (type 'help' for syntax, leave empty to not add TTL)")
		};

		// TODO: PrimitiveDateTime saves my local time? converting to utc will cause problems
		// TODO: parsing from utc could also cause problems

		let pretty_ttl = match ttl {
			Some(ttl) => Some(ttl.format(&USER_PROMPT_DATE_FORMAT)?),
			None => None,
		};

		let mut parsed_input = None;
		let input = self.prompter.input_with_validation(
			prompt,
			pretty_ttl.as_ref().map(std::convert::AsRef::as_ref),
			|new_ttl: &String | -> Result<(), &'static str> {
				let new_ttl = new_ttl.trim();
				if new_ttl.is_empty() {
					Ok(())
				} else if let Ok(std_duration) = parse_duration(new_ttl) {
					let Ok(time_duration) = time::Duration::try_from(std_duration) else {
						return Err("Something went wrong with the conversion the duration");
					};
					let Ok(now) = OffsetDateTime::now_local() else {
						return Err("Can't get local time, try to set TTL with timestamp syntax instead of duration");
					};
					let now = PrimitiveDateTime::new(now.date(), now.time());
					let Some(result) = now.checked_add(time_duration) else {
						return Err("The given duration is too far into the future");
					};
					parsed_input = Some(result);
					Ok(())
				} else if let Ok(timestamp) = parse_rfc3339_weak(new_ttl) {
					let timestamp = OffsetDateTime::from(timestamp);
					parsed_input = Some(PrimitiveDateTime::new(timestamp.date(), timestamp.time()));
					Ok(())
				} else {
					Err(HELP_SYNTAX)
				}
			})?;

		let new_ttl = match (input, parsed_input) {
			(Input::NoneEmpty(_), Some(new_ttl)) => new_ttl,
			(Input::Empty, None) => return Ok(()),
			(Input::Empty, Some(_)) | (Input::NoneEmpty(_), None) => unreachable!("Invalid state due to the validator"),
		};

		*ttl = Some(new_ttl);

		Ok(())
	}
}

impl <T: Theme> Wizard<'_, T> {
	fn get_reverse_orch_aliases(&self) -> ContainerMapping {
		Wizard::<T>::reverse_orch_aliases(&self.config.orchestration.aliases)
	}

	/// Generate a mapping from container names to their aliases,
	/// according to the current config.
	fn reverse_orch_aliases(aliases: &AliasToContainer) -> ContainerMapping {
		aliases
			.iter()
			.map(|(alias, container_name)| (alias.clone(), container_name.clone()))
			.fold(ContainerMapping { containers: IndexMap::new(), aliases: HashSet::new() }, |mut result, (alias, ContainerName { name: container_name, ttl })| {
				let alias = Rc::from(alias);
				result.containers.entry(Rc::from(container_name)).or_default().insert(ContainerAlias { name: Rc::clone(&alias), ttl });
				result.aliases.insert(alias);
				result
			})
	}

	fn set_restore_orch_aliases(&mut self, reversed_aliases: ContainerToAlias) {
		self.config.orchestration.aliases = Wizard::<T>::restore_orch_aliases(reversed_aliases);
	}

	/// Reverse a mapping from container names to their aliases,
	/// back to the config format.
	fn restore_orch_aliases(reversed_aliases: ContainerToAlias) -> AliasToContainer {
		reversed_aliases
			.into_iter()
			.fold(HashMap::new(), |mut result, (container_name, aliases)| {
				let container_name = container_name.to_string();
				for ContainerAlias { name: alias, ttl } in aliases {
					match result.entry(alias.to_string()) {
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
}

impl Run for Command {
	fn run(self, global_options: &GlobalOptions) -> Result<()> {
		if let Some(config) = Wizard::new(global_options, Self::get_theme()).run()? {
			FileHandler::save(&config, global_options)?;
		}
		Ok(())
	}
}
