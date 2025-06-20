use crate::prelude::PACKAGE_NAME;
pub use docker_compose_types::Compose as DockerComposeType;

const CONFIG_FILE_NAME: &str = "current_state_dryrun";

#[allow(dead_code)]
pub trait HandleConfig {
	fn load() -> Self;
	fn store(&self);
	fn get_container_names<T>(_to_ignore_warnings: T) -> Vec<String>;
}

impl HandleConfig for DockerComposeType {
	fn load() -> Self {
		match confy::load(PACKAGE_NAME, CONFIG_FILE_NAME) {
			Ok(current_state) => current_state,
			Err(error) => panic!("Failed to parse CurrentState config, error: {error}"),
		}
	}

	fn store(&self) {
		match confy::store(PACKAGE_NAME, CONFIG_FILE_NAME, self) {
			Ok(()) => (),
			Err(error) => panic!("Failed to store CurrentState config, error: {error}"),
		}
	}

	fn get_container_names<T>(_to_ignore_warnings: T) -> Vec<String> {
		Self::load().services.0
			.into_iter()
			.map(|x| x.0)
			.collect()
	}
}