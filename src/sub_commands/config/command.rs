use std::rc::Rc;
use humantime::{parse_duration, parse_rfc3339_weak};
use anyhow::Result;
use clap::Args;
use time::{OffsetDateTime, PrimitiveDateTime};
use dialoguer::theme::{ColorfulTheme, Theme};
use lazy_static::lazy_static;
use strum::{EnumString, FromRepr, VariantNames};
use super::{
	edit_structure::{AliasEntry, ContainerMapping, RcAlias, RcContainerName, Ttl},
	file_handler::FileHandler,
	structure::{Config, Kubernetes, OrchestrationType},
};
use crate::{
	Run,
	GlobalOptions,
	prompts::{input::Input, multi_select::ToDefaults, prompter::Prompter, selection::{from_repr, Options, Selection, SelectionReturn}},
};

lazy_static! {
	static ref USER_PROMPT_DATE_FORMAT: Vec<time::format_description::BorrowedFormatItem<'static>> = {
        time::format_description::parse("[year repr:full]-[month]-[day] [hour]:[minute]:[second padding:zero]").unwrap()
    };
}

#[derive(Args, Debug)]
#[command(about = "Modify the default behavior of the script.")]
#[command(long_about = "Modify the default behavior of the script.
Will enter into interactive CLI that will allow you to edit your config file.")]
pub struct Command;

// Utils for Interactive Shell
impl Command {
	fn get_theme() -> ColorfulTheme {
		ColorfulTheme::default()
	}
}

struct Wizard<'cfg, T: Theme> {
	prompter: Prompter<T>,
	#[allow(dead_code)]
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
			let containers = container_to_alias.get_display_container_names_as_rc();
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
					let is_need_to_be_removed = self.menu_7_edit_container(&mut container_to_alias, Rc::clone(container), SelectionReturn::default())?;
					if is_need_to_be_removed {
						container_to_alias.shift_remove_index_container_name(options_index);
						if containers.len() == 1 { // It was the last container
							selected = SelectionReturn::Return;
						} else if options_index == containers.len() -1 {
							selected = SelectionReturn::Selection(Selection { vec_index: 1, options_index: options_index - 1 });
						}
					}
				},
				SelectionReturn::Return => {
					self.set_restore_orch_aliases(container_to_alias);
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
			} else if container_to_alias.contains_container_name(container_name) {
				Err(format!("The container name {container_name:?} already exists"))
			} else {
				Ok(())
			}
		})?;

		let result = match input {
			Input::NoneEmpty(container_name) => {
				let container_name = Rc::from(container_name);
				if self.menu_16_new_container_new_alias(Rc::clone(&container_name), container_to_alias)? {
					!self.menu_7_edit_container(container_to_alias, container_name, SelectionReturn::default())?
				} else {
					false
				}
			},
			Input::Empty => false,
		};

		Ok(result)
	}

	fn prompt_new_alias(&self, container_name: &str, container_to_alias: &ContainerMapping) -> Result<Option<(RcAlias, Ttl)>> {
		let input = self.prompter.input_with_validation(format!("Choose new alias for {container_name:?} (leave empty to not add)"), None,
		|alias: &String | -> Result<(), String> {
			let alias = alias.trim();
			if alias.is_empty() {
				Ok(())
			} else if container_to_alias.contains_alias(alias) {
				Err(format!("The alias {alias:?} already exists"))
			} else {
				Ok(())
			}
		})?;

		let alias: RcAlias = match input {
			Input::NoneEmpty(alias) => Rc::from(alias),
			Input::Empty => return Ok(None),
		};

		let ttl = self.prompt_ttl(container_name, alias.as_ref(), None)?;

		Ok(Some((alias, ttl)))
	}

	fn menu_6_new_alias(&self, container_name: &str, container_to_alias: &mut ContainerMapping) -> Result<()> {
		if let Some((alias, ttl)) = self.prompt_new_alias(container_name, container_to_alias)? {
			container_to_alias.insert_new_alias(container_name, alias, ttl);
		}

		Ok(())
	}

	/// # Returns
	/// If `true`, the container was added.
	fn menu_16_new_container_new_alias(&self, container_name: Rc<str>, container_to_alias: &mut ContainerMapping) -> Result<bool> {
		if let Some((alias, ttl)) = self.prompt_new_alias(&container_name, container_to_alias)? {
			container_to_alias.insert_new_container_and_alias(container_name, alias, ttl);
			Ok(true)
		} else {
			Ok(false)
		}
	}

	/// # Returns
	/// If `true`, the container needs to be removed
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
						EditAlias::Add => self.menu_6_new_alias(&container_name, container_to_alias)?,
						EditAlias::Remove => self.menu_8_multi_remove_aliases(&container_name, container_to_alias)?,
					}
				},
				SelectionReturn::Selection(Selection { vec_index: 1, options_index: 0 }) => {
					if let Some(new_container_name) = self.menu_11_rename_container(&container_name, container_to_alias)? {
						container_name = new_container_name;
					}
				},
				SelectionReturn::Selection(Selection { vec_index: 1, options_index: 1 }) => {
					if self.menu_12_confirm_remove_container(&container_name)? {
						return Ok(true);
					}
				},
				SelectionReturn::Selection(Selection { vec_index: 2, options_index }) => {
					let alias_entry = container_to_alias.entry_alias_index(Rc::clone(&container_name), options_index);
					let is_need_to_be_removed = self.menu_9_edit_alias(alias_entry, SelectionReturn::default())?;
					if is_need_to_be_removed {
						container_to_alias.shift_remove_index_alias(&container_name, options_index);
						if aliases.len() == 1 { // It was the last alias
							selected = SelectionReturn::Return;
						} else if options_index == aliases.len() -1 {
							selected = SelectionReturn::Selection(Selection { vec_index: 2, options_index: options_index - 1 });
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

		container_to_alias.shift_remove_index_aliases(container_name, selected.into_iter().rev());
		
		Ok(())
	}

	/// # Returns
	/// If `true`, the alias needs to be removed.
	fn menu_9_edit_alias(&self, mut alias_entry: AliasEntry<'_>, mut select: SelectionReturn) -> Result<bool> {
		#[derive(EnumString, FromRepr, VariantNames)]
		#[repr(u8)]
		enum EditAlias {
			#[strum(serialize = "✏️  Rename alias")]
			Rename,
			#[strum(serialize = "❌ Remove alias")]
			Remove,
		}

		loop {
			let has_ttl = alias_entry.has_ttl();
			let edit_ttl = if has_ttl {
				vec!["✏️  Edit TTL", "❌ Remove TTL"]
			} else {
				vec!["➕ Add TTL"]
			};
			let options = Options::with_capacity(EditAlias::VARIANTS.len() + edit_ttl.len() + 1)
					.insert_str_refs(EditAlias::VARIANTS)
					.insert_str_refs(&edit_ttl)
					.insert_return()
					.set_selection(&select);

			let mut selected = self.prompter.select_with_return(format!("Editing {:?} for {:?}", alias_entry.get_name(), alias_entry.get_container_name()), &options)?;
			match (has_ttl, selected) {
				(_, SelectionReturn::Selection(Selection { vec_index: 0, options_index })) => {
					match from_repr!(EditAlias, options_index) {
						EditAlias::Rename => self.menu_10_rename_alias(&mut alias_entry)?,
						EditAlias::Remove => return Ok(true),
					}
				},
				(_, SelectionReturn::Selection(Selection { vec_index: 1, options_index: 0 })) => self.menu_15_edit_ttl(&mut alias_entry)?,
				(true, SelectionReturn::Selection(Selection { vec_index: 1, options_index: 1 })) => {
					alias_entry.remove_ttl();
					selected = SelectionReturn::Selection(Selection { vec_index: 1, options_index: 0 });
				},
				(_, SelectionReturn::Return) => return Ok(false),

				(false, SelectionReturn::Selection(Selection { vec_index: 1, options_index: 1 })) => unreachable!("Can't remove TTL if it doesn't have TTL"),
				(true, SelectionReturn::Selection(Selection { vec_index: 1, options_index: _ })) => unreachable!("edit_ttl with ttl has only 2 options"),
				(false, SelectionReturn::Selection(Selection { vec_index: 1, options_index: _ })) => unreachable!("edit_ttl without ttl has only 1 option"),
				(_, SelectionReturn::Selection(Selection { vec_index: _, options_index: _ })) => unreachable!("options has only 3 elements"),
			}
			select = selected;
		}
	}

	fn menu_10_rename_alias(&self, alias: &mut AliasEntry<'_>) -> Result<()> {
		let mut is_same_alias_name = false;
		let input = self.prompter.input_with_validation(
			format!("Choose new alias for {:?} of {:?} (leave empty to not rename)", alias.get_name(), alias.get_container_name()),
			Some(alias.get_name()),
			|new_alias: &String | -> Result<(), String> {
				let new_alias = new_alias.trim();
				if new_alias.is_empty() {
					Ok(())
				} else if alias.get_name() == new_alias {
					is_same_alias_name = true;
					Ok(())
				} else if alias.contains_alias(new_alias) {
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

		alias.rename(new_alias);

		Ok(())
	}

	fn menu_11_rename_container(&self, container_name: &str, container_to_alias: &mut ContainerMapping) -> Result<Option<RcContainerName>> {
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
				} else if container_to_alias.contains_container_name(new_container_name) {
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

		container_to_alias.rename_container_name(container_name, &new_container_name);

		Ok(Some(new_container_name))
	}

	fn menu_12_confirm_remove_container(&self, container_name: &str) -> Result<bool> {
		self.prompter.confirm(format!("Are you sure you want to remove {container_name:?}?"))
	}

	fn menu_13_multi_remove_containers(&self, container_to_alias: &mut ContainerMapping) -> Result<()> {
		let mut defaults = None;
		
		let selected = loop {
			let containers = container_to_alias.get_display_container_names_as_str();
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

		container_to_alias.shift_remove_index_container_names(selected.into_iter().rev());
		
		Ok(())
	}

	fn menu_14_confirm_remove_containers(&self) -> Result<bool> {
		self.prompter.confirm("Are you sure you want to remove those containers?")
	}

	fn prompt_ttl(&self, container_name: &str, alias: &str, previous_ttl: Option<&PrimitiveDateTime>) -> Result<Option<PrimitiveDateTime>> {
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

		let prompt = if previous_ttl.is_some() {
			format!("Editing TTL for {alias:?} of {container_name:?} (type 'help' for syntax, leave empty to not change the TTL)")
		} else {
			format!("Adding TTL for {alias:?} of {container_name:?} (type 'help' for syntax, leave empty to not add TTL)")
		};

		// TODO: PrimitiveDateTime saves my local time? converting to utc will cause problems
		// TODO: parsing from utc could also cause problems

		let mut parsed_input = None;
		let input = self.prompter.input_with_validation(
			prompt,
			previous_ttl
				.map(|ttl| ttl.format(&USER_PROMPT_DATE_FORMAT))
				.transpose()?
				.as_ref()
				.map(std::convert::AsRef::as_ref),
			|new_ttl: &String | -> Result<(), &'static str> {
				let new_ttl = new_ttl.trim();
				if new_ttl.is_empty() {
					Ok(())
				} else if let Ok(std_duration) = parse_duration(new_ttl) {
					let Ok(time_duration) = time::Duration::try_from(std_duration) else {
						return Err("Something went wrong with the conversion the duration");
					};
					let Ok(now) = OffsetDateTime::now_local() else {
						return Err("Can't get local time");
					};
					let now = PrimitiveDateTime::new(now.date(), now.time());
					let Some(result) = now.checked_add(time_duration) else {
						return Err("The given duration is too far into the future");
					};

					parsed_input = Some(result);
					Ok(())
				} else if let Ok(timestamp) = parse_rfc3339_weak(new_ttl) {
					let timestamp = OffsetDateTime::from(timestamp);
					let timestamp = PrimitiveDateTime::new(timestamp.date(), timestamp.time());

					let Ok(now) = OffsetDateTime::now_local() else {
						return Err("Can't get local time");
					};
					let now = PrimitiveDateTime::new(now.date(), now.time());
					if timestamp <= now {
						return Err("The given timestamp is in the past, use one in the future");
					}

					parsed_input = Some(timestamp);
					Ok(())
				} else {
					Err(HELP_SYNTAX)
				}
			})?;

		let result = match (input, parsed_input) {
			(Input::NoneEmpty(_), Some(new_ttl)) => Some(new_ttl),
			(Input::Empty, None) => None,
			(Input::Empty, Some(_)) | (Input::NoneEmpty(_), None) => unreachable!("Invalid state due to the validator"),
		};

		Ok(result)
	}

	fn menu_15_edit_ttl(&self, alias_entry: &mut AliasEntry<'_>) -> Result<()> {
		if let Some(new_ttl) = self.prompt_ttl(alias_entry.get_container_name(), alias_entry.get_name(), alias_entry.get_ttl())? {
			alias_entry.set_ttl(new_ttl);
		}

		Ok(())
	}
}

impl <T: Theme> Wizard<'_, T> {
	fn get_reverse_orch_aliases(&self) -> ContainerMapping {
		ContainerMapping::new(&self.config.orchestration.aliases)
	}

	fn set_restore_orch_aliases(&mut self, reversed_aliases: ContainerMapping) {
		self.config.orchestration.aliases = reversed_aliases.restore();
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
