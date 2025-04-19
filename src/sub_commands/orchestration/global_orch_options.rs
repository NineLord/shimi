use tap::prelude::*;
use anyhow::{anyhow, Result};
use log::trace;
use indexmap::{IndexSet, indexset};
use crate::{utils::ExitError, GlobalOptions, sub_commands::config::OrchestrationType};

#[derive(Debug)]
pub struct GlobalOrchOptions<'cfg> {
	global_options: &'cfg GlobalOptions,
	orchestration_type: Option<OrchestrationType>,
	name_space: Option<String>,
}

impl <'cfg> GlobalOrchOptions<'cfg> {
	pub const fn new(global_options: &'cfg GlobalOptions, orchestration_type: Option<OrchestrationType>, name_space: Option<String>) -> Self {
		Self {
			global_options,
			orchestration_type,
			name_space
		}
	}
}

impl GlobalOrchOptions<'_> {
	pub const fn get_orchestration_type(&self) -> OrchestrationType {
		match self.orchestration_type {
			Some(variant) => variant,
			None => self.global_options.config.orchestration.variant,
		}
	}
}


impl <'cfg> GlobalOrchOptions<'cfg> {
	pub fn get_name_space(&self) -> Result<&String> {
		match (&self.name_space, &self.global_options.config.orchestration.kubernetes) {
			(Some(name_space), _) => Ok(name_space),
			(None, Some(kubernetes)) => Ok(&kubernetes.name_space),
			(None, None) => Err(anyhow!("Namespace wasn't set in the config nor given via optional argument")),
		}
	}

	pub fn add_name_space<'args>(&'cfg self, arguments: &'args mut Vec<&'cfg str>) {
		let name_space: &'cfg String = match self.get_name_space() {
			Ok(name_space) => name_space,
			Err(error) => ExitError::BadArgument.exit(error),
		};

		arguments.push("--namespace");
		arguments.push(name_space);
	}
}

pub enum IsExactMatch {
	/// Won't try to convert the `input` using aliases.
	Yes,
	/// If the `input` is alias, it will be converted.
	No(IsTryGetMatch),
}

pub enum IsTryGetMatch {
	/// Won't try to find a matching container.
	No,
	/// Will try to find a matching container,
	/// from the list of sources.
	/// The order of list represent priority order,
	/// once a match is found, it won't go over the other sources.
	/// # Panics
	/// * If the given list is empty.
	Yes(IndexSet<MatchSource>)
}

#[derive(Hash, PartialEq, Eq)]
pub enum MatchSource {
	/// Will try to find a match against existing containers.
	ExitingContainers {
		/// If true, will **also** try to find a match in:
		/// * `docker-compose` - The `stopped`/`killed` state containers.
		/// * `kubernetes` - The other name spaces.
		is_all_containers: bool,
	},
	/// Will try to find a match against the config file:
	/// * `docker-compose` - The `docker-compose.yml` file.
	/// * `kubernetes` - The `<TODO>` files.
	Config,
}

impl Default for IsTryGetMatch {
	fn default() -> Self {
		Self::Yes(indexset!{
			MatchSource::ExitingContainers { is_all_containers: false }
		})
	}
}

/*
	// Shaked-TODO:
	1. Update the docs aboves
	2. impl this options, while ignore this specific one
	3. disable up, down, reset, ip, port_forward commands.
	4. Add exact match opt.
	5. public first exe and test it.
*/

impl GlobalOrchOptions<'_> {
	/// # Params
	/// * `input` - The name of the container, could be an alias or substring of the full container name.
	/// * `options` - Additional settings.
	pub fn get_container_name(&self, input: &str, options: IsExactMatch) -> Result<String> {
		// Shaked-TODO: can optimize this to also separate the instance number of the container and analyze it
		#[derive(Debug)]
		struct ContainerName {
			original: String,
			abbreviation: String,
		}

		let container_names = self.get_container_names(false)?
			.into_iter()
			.map(|container_name| ContainerName {
				abbreviation: container_name.to_lowercase()
					.tap_mut(|name| name.retain(char::is_alphanumeric)),
				original: container_name,
			})
			.collect::<Vec<ContainerName>>()
			.tap_mut(|container_names| container_names.sort_unstable_by(
				|cn1, cn2| cn1.abbreviation.cmp(&cn2.abbreviation)
			));
		
		trace!("get_container_name :: Sorted container names: {container_names:#?}");

		let name = self.global_options.config.orchestration.aliases
			.get(input)
			.map_or(input, |alias| alias.as_ref())
			.to_lowercase()
			.tap_mut(|name| name.retain(char::is_alphanumeric));
		
		trace!("get_container_name :: Aliases name: {name:?}");

		let result = container_names
			.into_iter()
			.find(|ContainerName { original: _, abbreviation }| abbreviation.contains(&name))
			.map(|ContainerName { original, abbreviation: _ }| original);
	
		trace!("get_container_name :: Chosen container: {result:#?}");
		
		let Some(result) = result else {
			ExitError::BadArgument.exit(format!("Couldn't find container with the name {input:?} or alias for it"));
		};

		Ok(result)
	}
}