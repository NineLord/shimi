use std::{fmt::Display, rc::Rc, hash::Hash, iter};
use log::warn;
use hashbrown::{HashMap, hash_map::Entry as HashbrownMapEntry, Equivalent as EquivalentHashbrown};
use indexmap::{set::MutableValues, Equivalent as EquivalentIndexMap, IndexMap, IndexSet};
use time::PrimitiveDateTime;
use lazy_static::lazy_static;
use super::structure::{AliasToContainer, ContainerName};

lazy_static! {
    static ref CONTAINER_ALIAS_DATE_FORMAT: Vec<time::format_description::BorrowedFormatItem<'static>> = {
        time::format_description::parse("[day]/[month]/[year repr:last_two] [hour]:[minute]").unwrap()
    };
}

pub type RcContainerName = Rc<str>;
pub type RcAlias = Rc<str>;
pub type Ttl = Option<PrimitiveDateTime>;
pub type TtlRef<'a> = Option<&'a PrimitiveDateTime>;

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
pub type Aliases = HashMap<RcAlias, RcContainerName>;
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
			.fold(Self { containers: IndexMap::new(), aliases: HashMap::new() }, |mut result, (alias, ContainerName { name: container_name, ttl })| {
				let alias = Rc::from(alias);
				let container_name = Rc::from(container_name);
				result.containers.entry(Rc::clone(&container_name)).or_default().insert(ContainerAlias { name: Rc::clone(&alias), ttl });
				result.aliases.insert(alias, container_name);
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
	pub fn contains_container_name<C: ?Sized + Hash + EquivalentIndexMap<Rc<str>>>(&self, container_name: &C) -> bool {
		self.containers.contains_key(container_name)
	}

	pub fn get_container_name_for_alias<A: ?Sized + Hash + EquivalentHashbrown<Rc<str>>>(&self, alias: &A) -> Option<&RcContainerName> {
		self.aliases.get(alias)
	}
}

impl ContainerMapping {
	/// # Panics
	/// * If the given `alias` already exists.
	pub fn insert_new_container_and_alias(&mut self, container_name: RcContainerName, alias: RcAlias, ttl: Ttl) {
		let container_name_cloned = Rc::clone(&container_name);
		let aliases = self.containers.entry(container_name).or_default();
		Self::insert_new_alias_helper(&mut self.aliases, aliases, alias, container_name_cloned, ttl);
	}

	/// # Panics
	/// * If the given `container_name` doesn't exists.
	/// * If the given `alias` already exists.
	pub fn insert_new_alias(&mut self, container_name: &str, alias: RcAlias, ttl: Ttl) {
		let (_, container_name, aliases) = self.containers.get_full_mut(container_name)
			.expect("The given container_name doesn't exists");
		Self::insert_new_alias_helper(&mut self.aliases, aliases, alias, Rc::clone(container_name), ttl);
	}

	/// # Panics
	/// * If the given `alias` already exists.
	fn insert_new_alias_helper(total_aliases: &mut Aliases, container_aliases: &mut IndexSet<ContainerAlias>, alias: RcAlias, container_name: RcContainerName, ttl: Ttl) {
		let is_new_alias = total_aliases.insert(Rc::clone(&alias), container_name);
		debug_assert!(is_new_alias.is_none(), "The given alias already exists");
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
		self.shift_remove_index_aliases(container_name, iter::once(alias_index));
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
			let previous_container_name = self.aliases.remove(&previous_alias.name);
			debug_assert!(previous_container_name.is_some(), "The given alias existed in the self.containers but not in the self.aliases");
		}
	}
}

impl ContainerMapping {
	/// # Panics
	/// * If the given `container_name` isn't valid index.
	pub fn shift_remove_index_container_name(&mut self, container_name: usize) {
		self.shift_remove_index_container_names(iter::once(container_name));
	}

	/// # Panics
	/// * If the given `container_names` doesn't contain valid indexes.
	pub fn shift_remove_index_container_names<I: Iterator<Item=usize>>(&mut self, container_names: I) {
		for container_name_index in container_names {
			let (_container_name, aliases) = self.containers.shift_remove_index(container_name_index)
				.expect("The given index must be valid");
			for alias in aliases {
				let previous_container_name = self.aliases.remove(&alias.name);
				debug_assert!(previous_container_name.is_some(), "The given alias existed in the self.containers but not in the self.aliases");
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

	#[inline]
	pub fn get_container_name(&self) -> &str {
		self.container_name.as_ref()
	}

	#[inline]
	pub fn get_name(&self) -> &str {
		self.alias.name.as_ref()
	}

	pub fn get_container_name_for_alias<A: ?Sized + Hash + EquivalentHashbrown<Rc<str>>>(&self, alias: &A) -> Option<&RcContainerName> {
		self.aliases.get(alias)
	}

	pub fn get_ttl(&self) -> TtlRef<'_> {
		self.alias.ttl.as_ref()
	}

	pub fn set_ttl(&mut self, new: PrimitiveDateTime) {
		self.alias.ttl = Some(new);
	}

	pub fn remove_ttl(&mut self) {
		self.alias.ttl = None;
	}

	pub fn rename(&mut self, new: RcAlias) {
		let container_name = self.aliases.remove(&self.alias.name)
			.expect("Can't rename an alias that doesn't exists");
		self.alias.name = new;
		self.aliases.insert(Rc::clone(&self.alias.name), container_name);
	}
}