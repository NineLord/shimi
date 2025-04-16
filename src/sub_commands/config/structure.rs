use serde::{Serialize, Deserialize};
use strum::{EnumString, FromRepr, VariantNames};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub version: String,
	pub orchestration: Orchestration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Orchestration {
	pub r#type: OrchestrationType,
	pub kubernetes: Option<Kubernetes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Kubernetes {
	pub name_space: String,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Serialize, Deserialize, EnumString, FromRepr, VariantNames)]
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
				r#type: OrchestrationType::DockerCompose,
				kubernetes: None,
			},
		}
	}
}
