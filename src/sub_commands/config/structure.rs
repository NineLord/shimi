use serde::{Serialize, Deserialize};
use strum::{EnumString, FromRepr, VariantNames};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub version: String,
	pub orchestration: Orchestration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orchestration {
	pub variant: OrchestrationType,
	pub kubernetes: Option<Kubernetes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kubernetes {
	pub name_space: String,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, Serialize, Deserialize, EnumString, FromRepr, VariantNames)]
#[repr(u8)]
pub enum OrchestrationType {
	#[strum(serialize = "Docker-Compose")]
	DockerCompose,
	#[strum(serialize = "Kubernetes")]
	Kubernetes,
}

impl OrchestrationType {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

impl Config {
	#[must_use]
	pub const fn default(version: String) -> Self {
		Self {
			version,
			orchestration: Orchestration {
				variant: OrchestrationType::DockerCompose,
				kubernetes: None,
			},
		}
	}
}
