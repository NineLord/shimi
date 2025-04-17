use anyhow::{anyhow, Result};
use crate::{utils::ExitError, sub_commands::config::{Config, OrchestrationType}};

#[derive(Debug)]
pub struct GlobalOrchOptions<'cfg> {
	config: &'cfg Config,
	orchestration_type: Option<OrchestrationType>,
	name_space: Option<String>,
}

impl <'cfg> GlobalOrchOptions<'cfg> {
	pub const fn new(config: &'cfg Config, orchestration_type: Option<OrchestrationType>, name_space: Option<String>) -> Self {
		Self {
			config,
			orchestration_type,
			name_space
		}
	}
}

impl GlobalOrchOptions<'_> {
	pub const fn get_orchestration_type(&self) -> OrchestrationType {
		match self.orchestration_type {
			Some(variant) => variant,
			None => self.config.orchestration.variant,
		}
	}
}


impl <'cfg> GlobalOrchOptions<'cfg> {
	pub fn get_name_space(&self) -> Result<&String> {
		match (&self.name_space, &self.config.orchestration.kubernetes) {
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