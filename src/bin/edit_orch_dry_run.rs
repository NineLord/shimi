use tap::prelude::*;
#[cfg(feature = "dry_run")]
use shimi::sub_commands::orchestration::dry_run::{DockerComposeType, HandleConfig};

fn main() {
	#[cfg(not(feature = "dry_run"))]
	panic!("Suppose to be used only in with '--features dry_run'");

	#[cfg(feature = "dry_run")]
	if false {
		DockerComposeType::new()
	} else {
		DockerComposeType::load()
	}
		.tap_mut(|docker_compose| {
			let services = &mut docker_compose.services.0;
			services.insert(String::from("avatar"), None);
			services.insert(String::from("you_tube_2"), None);
			services.insert(String::from("path-of-exile-1"), None);
			services.insert(String::from("you_tube_1"), None);
			services.insert(String::from("last-epoch"), None);
			services.insert(String::from("you_tube_3"), None);
			services.insert(String::from("path-of-exile-2"), None);
			services.insert(String::from("spongebob"), None);
			services.insert(String::from("facebook-jfdksalfhdka"), None);
			services.insert(String::from("facebook-djrieoruaiea"), None);
			services.insert(String::from("facebook-fjdasklfjsad"), None);
		})
		.store();

	println!("Done");
}