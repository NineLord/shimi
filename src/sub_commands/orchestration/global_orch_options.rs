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

//#region get_container_name
#[derive(Debug)]
pub enum IsExactMatch {
	/// Won't try to convert the `input` using aliases.
	Yes,
	/// If the `input` is alias, it will be converted.
	No(IsTryGetMatch),
}

#[derive(Debug)]
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

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MatchSource {
	/// Will try to find a match against existing containers.
	ExitingContainers {
		/// If true, will **also** try to find a match in:
		/// * `docker-compose` - The `stopped`/`killed` state containers.
		/// * `kubernetes` - The other name spaces.
		is_all: bool,
	},
	/// Will try to find a match against the config file:
	/// * `docker-compose` - The `docker-compose.yml` file.
	/// * `kubernetes` - The `<TODO>` files.
	Config,
}

impl Default for IsTryGetMatch {
	fn default() -> Self {
		Self::Yes(indexset!{
			MatchSource::ExitingContainers { is_all: false }
		})
	}
}

/*
	// Shaked-TODO:
	4. Add exact match opt.
	5. public first exe and test it.
*/

#[derive(Debug)]
struct ContainerName {
	original: String,
	abbreviation: String,
}

impl GlobalOrchOptions<'_> {
	/// # Params
	/// * `input` - The name of the container, could be an alias or substring of the full container name.
	/// * `options` - Additional settings.
	pub fn get_container_name(&self, input: String, is_exact_match: IsExactMatch) -> Result<String> {
		// Shaked-TODO: can optimize this to also separate the instance number of the container and analyze it
		trace!("get_container_name :: Options: {is_exact_match:#?}");

		let is_try_get_match = match is_exact_match {
			IsExactMatch::Yes => {
				trace!("get_container_name :: result={input:?}");
				return Ok(input);
			},
			IsExactMatch::No(is_try_get_match) => is_try_get_match,
		};

		let sources = match is_try_get_match {
			IsTryGetMatch::No => {
				let result = self.convert_to_alias(input);
				trace!("get_container_name :: result={result:?}");
				return Ok(result);
			},
			IsTryGetMatch::Yes(sources) => sources,
		};

		let matching_alias = {
			let alias = self.convert_to_alias(input.clone());
			let result = Self::prepare_for_matching(&alias);
			trace!("prepare_for_matching :: from {alias:?} to {result:?}");
			result
		};

		for source in sources {
			let result = source.to_container_names(self)?
				.into_iter()
				.find(|ContainerName { original: _, abbreviation }| abbreviation.contains(&matching_alias))
				.map(|ContainerName { original, abbreviation: _ }| original);


			if let Some(result) = result {
				trace!("get_container_name :: result={result:?}");
				return Ok(result);
			}
		}

		ExitError::BadArgument.exit(format!("Couldn't find container with the name {input:?} or alias for it"));
	}

	/// Convert the input to his alias, or defaults back to the input.
	/// # Params
	/// * `input` - The name that going to be converted.
	fn convert_to_alias(&self, input: String) -> String {
		let alias = self.global_options.config.orchestration.aliases.get(&input);
		trace!("convert_to_alias :: from {input:?} to {alias:?}");
		alias
			.cloned()
			.unwrap_or(input)
	}

	/// Convert the input to lowercase with only numbers.
	/// # Params
	/// * `input` - The name that going to be converted.
	fn prepare_for_matching(input: &str) -> String {
		input
			.to_lowercase()
			.tap_mut(|name| name.retain(char::is_alphanumeric))
	}
}

impl MatchSource {
	fn to_container_names(self, global_options: &GlobalOrchOptions) -> Result<Vec<ContainerName>> {
		let container_names = match self {
			Self::ExitingContainers { is_all } => global_options.get_container_names(is_all)?,
			Self::Config => todo!("MatchSource::Config"),
		}
			.into_iter()
			.map(|container_name| ContainerName {
				abbreviation: GlobalOrchOptions::prepare_for_matching(&container_name),
				original: container_name,
			})
			.collect::<Vec<ContainerName>>()
			.tap_mut(|container_names| container_names.sort_unstable_by(
				|cn1, cn2| cn1.abbreviation.cmp(&cn2.abbreviation)
			));

		trace!("MatchSource::to_container_names :: source={self:?} ; container_names={container_names:#?}");
		Ok(container_names)
	}
}
//#endregion
