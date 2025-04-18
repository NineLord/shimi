use tap::prelude::*;
use anyhow::{anyhow, Result};
use log::trace;
use super::process_status::Arguments as process_status;
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

impl GlobalOrchOptions<'_> {
	pub fn get_container_name(&self, input: &str) -> Result<String> {
		#[derive(Debug)]
		struct ContainerName {
			original: String,
			abbreviation: String,
		}

		let container_names = process_status::get_container_names(self, false)?
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