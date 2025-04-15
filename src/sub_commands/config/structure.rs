use serde::{Serialize, Deserialize};
use strum::{EnumString, FromRepr, VariantNames};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub orchestration: Orchestration,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Serialize, Deserialize, EnumString, FromRepr, VariantNames)]
#[repr(u8)]
pub enum Orchestration {
	#[strum(serialize = "Docker-Compose")]
	DockerCompose,
	#[strum(serialize = "Kubernetes")]
	Kubernetes {
		name_space: String,
	},
}

impl Orchestration {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

impl Default for Config {
	fn default() -> Self {
		Self {
			orchestration: Orchestration::DockerCompose
		}
	}
}
