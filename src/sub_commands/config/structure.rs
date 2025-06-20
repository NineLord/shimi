use hashbrown::HashMap;
use serde::{Serialize, Deserialize};
use strum::{EnumString, FromRepr, VariantNames};
use time::PrimitiveDateTime;
use clap::ValueEnum;
use getset::{Getters, MutGetters, Setters};

pub type StrAlias = String;
pub type AliasToContainer = HashMap<StrAlias, ContainerName>;


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
pub(super) enum VersionChecked {
	#[serde(rename = "0")]
	Version0(Config),
	#[serde(other)]
	Unsupported,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Getters, MutGetters, Setters)]
pub struct Config { // No constructor for this class, since it should be make only via deserializing a config file.
	#[getset(get = "pub", get_mut = "pub", set = "pub")]
	orchestration: Orchestration,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

impl Default for OrchestrationType {
	fn default() -> Self {
		Self::DockerCompose
	}
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
