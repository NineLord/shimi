use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub orchestration: Orchestration,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Orchestration {
	DockerCompose,
	Kubernetes {
		name_space: String,
	},
}

impl Default for Config {
	fn default() -> Self {
		Self {
			orchestration: Orchestration::DockerCompose
		}
	}
}
