use std::{fmt::Display, rc::Rc, hash::Hash};
use anyhow::Result;
use log::warn;
use hashbrown::{HashMap, hash_map::Entry as HashbrownMapEntry, HashSet, Equivalent as EquivalentHashbrown};
use indexmap::{set::MutableValues, Equivalent as EquivalentIndexMap, IndexMap, IndexSet};
use time::PrimitiveDateTime;
use lazy_static::lazy_static;
use super::structure::{AliasToContainer, ContainerName};

lazy_static! {
    static ref CONTAINER_ALIAS_DATE_FORMAT: Vec<time::format_description::BorrowedFormatItem<'static>> = {
        time::format_description::parse("[day]/[month]/[year repr:last_two] [hour]:[minute]").unwrap()
    };

	static ref USER_PROMPT_DATE_FORMAT: Vec<time::format_description::BorrowedFormatItem<'static>> = {
        time::format_description::parse("[year repr:full]-[month]-[day] [hour]:[minute]:[second padding:zero]").unwrap()
    };
}

pub type RcContainerName = Rc<str>;
pub type RcAlias = Rc<str>;
pub type Ttl = Option<PrimitiveDateTime>;
pub type TtlRef<'a> = Option<&'a PrimitiveDateTime>;
pub type TtlMutRef<'a> = Option<&'a mut PrimitiveDateTime>;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ContainerAlias {
	pub name: RcAlias,
	pub ttl: Ttl,
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

pub type ContainerToAlias = IndexMap<RcContainerName, IndexSet<ContainerAlias>>;
pub type Aliases = HashSet<RcAlias>;
#[derive(Debug)]
pub struct ContainerMapping {
	containers: ContainerToAlias,
	aliases: Aliases,
}

impl ContainerMapping {
	/// Generate a mapping from container names to their aliases,
	/// according to the current config.
	pub fn new(aliases: &AliasToContainer) -> Self {
		aliases
			.iter()
			.map(|(alias, container_name)| (alias.clone(), container_name.clone()))
			.fold(Self { containers: IndexMap::new(), aliases: HashSet::new() }, |mut result, (alias, ContainerName { name: container_name, ttl })| {
				let alias = Rc::from(alias);
				result.containers.entry(Rc::from(container_name)).or_default().insert(ContainerAlias { name: Rc::clone(&alias), ttl });
				result.aliases.insert(alias);
				result
			})
	}

	/// Reverse a mapping from container names to their aliases,
	/// back to the config format.
	pub fn restore(self) -> AliasToContainer {
		self.containers
			.into_iter()
			.fold(HashMap::new(), |mut result, (container_name, aliases)| {
				let container_name = container_name.to_string();
				for ContainerAlias { name: alias, ttl } in aliases {
					match result.entry(alias.to_string()) {
						HashbrownMapEntry::Vacant(entry) => {
							entry.insert(ContainerName { name: container_name.clone(), ttl });
						},
						HashbrownMapEntry::Occupied(entry) => {
							warn!("The alias {0:?} points to two different container names: {1:?} and {2:?} ; Ignoring: {2:?}",
								entry.key(), entry.get(), ContainerName { name: container_name.clone(), ttl });
						},
					}
				}
				result
			})
	}
}

impl ContainerMapping {
	pub fn get_display_container_names_as_str(&self) -> Vec<&str> {
		self.containers.keys()
			.map(std::convert::AsRef::as_ref)
			.collect()
	}

	pub fn get_display_container_names_as_rc(&self) -> Vec<Rc<str>> {
		self.containers.keys()
			.map(Rc::clone)
			.collect()
	}

	/// # Panics
	/// * If the given `container_name` doesn't exists in the container mapping.
	pub fn get_display_aliases(&self, container_name: &str) -> Vec<String> {
		self.containers.get(container_name)
			.expect("The given container_name must already exists")
			.iter()
			.map(std::string::ToString::to_string)
			.collect()
	}
}

impl ContainerMapping {
	pub fn entry_alias_index(&mut self, container_name: Rc<str>, alias_index: usize) -> AliasEntry<'_> {
		let alias = self.containers.get_mut(&container_name)
			.expect("The given container_name must already exists")
			.get_index_mut2(alias_index)
			.expect("The given alias must already exists");
		AliasEntry { container_name, alias, aliases: &mut self.aliases }
	}
}

impl ContainerMapping {
	/// # Panics
	/// * If the given `container_name` doesn't exists.
	/// * If the given `alias_index` doesn't exists.
	pub fn get_alias_name(&self, container_name: &str, alias_index: usize) -> &str {
		self.containers.get(container_name)
			.expect("The given container_name must already exists")
			.get_index(alias_index)
			.expect("The given alias must already exists")
			.name
			.as_ref()
	}

	/// # Panics
	/// * If the given `container_name` doesn't exists.
	/// * If the given `alias_index` doesn't exists.
	pub fn get_mut_alias_ttl(&mut self, container_name: &str, alias_index: usize) -> TtlMutRef {
		self.containers.get_mut(container_name)
			.expect("The given container_name must already exists")
			.get_index_mut2(alias_index)
			.expect("The given alias must already exists")
			.ttl
			.as_mut()
	}
}

impl ContainerMapping {
	pub fn contains_container_name<C: ?Sized + Hash + EquivalentIndexMap<Rc<str>>>(&self, container_name: &C) -> bool {
		self.containers.contains_key(container_name)
	}

	pub fn contains_alias<A: ?Sized + Hash + EquivalentHashbrown<Rc<str>>>(&self, alias: &A) -> bool {
		self.aliases.contains(alias)
	}
}

impl ContainerMapping {
	/// # Panics
	/// * If the given `alias` already exists.
	pub fn insert_new_container_and_alias(&mut self, container_name: RcContainerName, alias: RcAlias, ttl: Ttl) {
		let aliases = self.containers.entry(container_name).or_default();
		Self::insert_new_alias_helper(&mut self.aliases, aliases, alias, ttl);
	}

	/// # Panics
	/// * If the given `container_name` doesn't exists.
	/// * If the given `alias` already exists.
	pub fn insert_new_alias(&mut self, container_name: &str, alias: RcAlias, ttl: Ttl) {
		let aliases = self.containers.get_mut(container_name)
			.expect("The given container_name doesn't exists");
		Self::insert_new_alias_helper(&mut self.aliases, aliases, alias, ttl);
	}

	/// # Panics
	/// * If the given `alias` already exists.
	fn insert_new_alias_helper(total_aliases: &mut Aliases, container_aliases: &mut IndexSet<ContainerAlias>, alias: RcAlias, ttl: Ttl) {
		let is_new_alias = total_aliases.insert(Rc::clone(&alias));
		debug_assert!(is_new_alias, "The given alias already exists");
		let is_new_alias = container_aliases.insert(ContainerAlias { name: alias, ttl });
		debug_assert!(is_new_alias, "The given alias already exists");
	}

	/// # Panics
	/// * If the given `new` already exists.
	/// * If the given `previous` doesn't exists.
	pub fn rename_container_name(&mut self, previous: &str, new: &RcContainerName) {
		let new_container_name_entry = self.containers.insert(Rc::clone(new), IndexSet::new());
		debug_assert!(new_container_name_entry.is_none(), "The given new already exists");
		match self.containers.swap_remove(previous) {
			Some(prev_aliases) => {
				let aliases = self.containers.get_mut(new)
					.expect("The new container name was just inserted");
				*aliases = prev_aliases;
			},
			None => unreachable!("The previous must already exists"),
		}
	}
}

impl ContainerMapping {
	/// # Panics
	/// * If the given `container_name` doesn't already exists.
	/// * If the given `alias_index` isn't valid index.
	pub fn shift_remove_index_alias(&mut self, container_name: &str, alias_index: usize) {
		self.shift_remove_index_aliases(container_name, std::iter::once(alias_index));
	}

	/// # Panics
	/// * If the given `container_name` doesn't already exists.
	/// * If the given `aliases_index` doesn't contain valid indexes.
	pub fn shift_remove_index_aliases<I: Iterator<Item=usize>>(&mut self, container_name: &str, aliases_index: I) {
		let aliases = self.containers.get_mut(container_name)
			.expect("The given container_name must already exists");

		for alias_index in aliases_index {
			let previous_alias = aliases.shift_remove_index(alias_index)
				.expect("The given index must be valid");
			let is_removed = self.aliases.remove(&previous_alias.name);
			debug_assert!(is_removed, "The given alias existed in the self.containers but not in the self.aliases");
		}
	}
}

impl ContainerMapping {
	/// # Panics
	/// * If the given `container_name` isn't valid index.
	pub fn shift_remove_index_container_name<I: Iterator<Item=usize>>(&mut self, container_name: usize) {
		self.shift_remove_index_container_names(std::iter::once(container_name));
	}

	/// # Panics
	/// * If the given `container_names` doesn't contain valid indexes.
	pub fn shift_remove_index_container_names<I: Iterator<Item=usize>>(&mut self, container_names: I) {
		for container_name_index in container_names {
			let (_container_name, aliases) = self.containers.shift_remove_index(container_name_index)
				.expect("The given index must be valid");
			for alias in aliases {
				let is_removed = self.aliases.remove(&alias.name);
				debug_assert!(is_removed, "The given alias existed in the self.containers but not in the self.aliases");
			}
		}
	}
}

pub struct AliasEntry<'a> {
	container_name: RcContainerName,
	alias: &'a mut ContainerAlias,
	aliases: &'a mut Aliases,
}

impl AliasEntry<'_> {
	#[inline]
	pub fn has_ttl(&self) -> bool {
		self.alias.ttl.is_some()
	}

	pub fn get_ttl_as_user_prompt(&self) -> Result<Option<String>> {
		// TODO: PrimitiveDateTime saves my local time? converting to utc will cause problems
		// TODO: parsing from utc could also cause problems
		let result = match self.alias.ttl {
			Some(ttl) => Some(ttl.format(&USER_PROMPT_DATE_FORMAT)?),
			None => None,
		};
		Ok(result)
	}

	// #[inline]
	// pub fn get_mut_ttl(&mut self) -> &mut Ttl {
	// 	&mut self.alias.ttl
	// }

	#[inline]
	pub fn get_container_name(&self) -> &str {
		self.container_name.as_ref()
	}

	#[inline]
	pub fn get_name(&self) -> &str {
		self.alias.name.as_ref()
	}

	pub fn contains_alias<A: ?Sized + Hash + EquivalentHashbrown<Rc<str>>>(&self, alias: &A) -> bool {
		self.aliases.contains(alias)
	}

	pub fn set_ttl(&mut self, new: PrimitiveDateTime) {
		self.alias.ttl = Some(new);
	}

	pub fn remove_ttl(&mut self) {
		self.alias.ttl = None;
	}

	pub fn rename(&mut self, new: RcAlias) {
		self.aliases.remove(&self.alias.name);
		self.alias.name = new;
		self.aliases.insert(Rc::clone(&self.alias.name));
	}
}