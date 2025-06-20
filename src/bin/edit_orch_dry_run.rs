use tap::prelude::*;
#[cfg(feature = "dry_run")]
use shimi::sub_commands::orchestration::dry_run::{DockerComposeType, HandleConfig, AddService, Scale, Stateful, NamingConvention};

fn main() {
	#[cfg(not(feature = "dry_run"))]
	panic!("Suppose to be used only in with '--features dry_run'");

	#[cfg(feature = "dry_run")]
	if true {
		DockerComposeType::new()
	} else {
		DockerComposeType::load()
	}
		.tap_mut(|docker_compose| {
			docker_compose.services.add_service(String::from("warframe"), None);
			docker_compose.services.add_service(String::from("hades"), Some(Scale::Stateless(2)));
			docker_compose.services.add_service(String::from("diablo"), Some(Scale::Stateful(Stateful {
				scale: 4, naming_convention: NamingConvention::UnderscoreNumber0
			})));
			docker_compose.services.add_service(String::from("bioShock"), Some(Scale::Stateful(Stateful {
				scale: 2, naming_convention: NamingConvention::UnderscoreNumber1
			})));
			docker_compose.services.add_service(String::from("bioShock-infinite"), None);
			docker_compose.services.add_service(String::from("portal"), Some(Scale::Stateful(Stateful {
				scale: 3, naming_convention: NamingConvention::DashHash // It's 3 because of `Portal Revolution`
			})));
			docker_compose.services.add_service(String::from("dontStarve"), None);
			docker_compose.services.add_service(String::from("dontStarveTogether"), None);
		})
		.store();

	println!("Done");
}