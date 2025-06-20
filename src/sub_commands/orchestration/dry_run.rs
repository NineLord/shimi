use serde::{Serialize, Deserialize};
use hashbrown::HashSet;
use crate::prelude::PACKAGE_NAME;

const CONFIG_FILE_NAME: &str = "current_state_dryrun";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CurrentState {
	pub containers: HashSet<String>,
}

#[allow(dead_code)]
impl CurrentState {
	pub fn load() -> Self {
		match confy::load(PACKAGE_NAME, CONFIG_FILE_NAME) {
			Ok(current_state) => current_state,
			Err(error) => panic!("Failed to parse CurrentState config, error: {error}"),
		}
	}

	pub fn store(&self) {
		match confy::store(PACKAGE_NAME, CONFIG_FILE_NAME, self) {
			Ok(()) => (),
			Err(error) => panic!("Failed to store CurrentState config, error: {error}"),
		}
	}

	pub fn get_container_names<T>(_to_ignore_warnings: T) -> Vec<String> {
		Self::load().containers.into_iter().collect()
	}
}