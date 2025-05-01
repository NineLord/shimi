use hashbrown::HashMap;
use serde::{Serialize, Deserialize};
use strum::{EnumString, FromRepr, VariantNames};
use time::PrimitiveDateTime;
use clap::ValueEnum;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub version: String,
	pub orchestration: Orchestration,
}

pub type StrAlias = String;
pub type AliasToContainer = HashMap<StrAlias, ContainerName>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orchestration {
	pub variant: OrchestrationType,
	pub kubernetes: Option<Kubernetes>,
	pub aliases: AliasToContainer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, FromRepr, VariantNames, ValueEnum)]
#[repr(u8)]
pub enum OrchestrationType {
	#[strum(serialize = "Docker-Compose")]
	DockerCompose,
	#[strum(serialize = "Kubernetes")]
	Kubernetes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kubernetes {
	pub name_space: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerName {
	pub name: String,
	pub ttl: Option<PrimitiveDateTime>
}

impl Config {
	#[must_use]
	pub fn default(version: String) -> Self {
		Self {
			version,
			orchestration: Orchestration {
				variant: OrchestrationType::DockerCompose,
				kubernetes: None,
				aliases: HashMap::default(),
			},
		}
	}
}
