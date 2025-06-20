use tap::prelude::*;
use hashbrown::HashSet;
#[cfg(feature = "dry_run")]
use shimi::sub_commands::orchestration::dry_run::CurrentState;

fn main() {
	#[cfg(not(feature = "dry_run"))]
	panic!("Suppose to be used only in with '--features dry_run'");

	#[cfg(feature = "dry_run")]
	#[allow(clippy::needless_update)]
	CurrentState {
		containers: HashSet::new()
			.tap_mut(|s| {
				s.insert(String::from("avatar"));
				s.insert(String::from("you_tube_2"));
				s.insert(String::from("path-of-exile-1"));
				s.insert(String::from("you_tube_1"));
				s.insert(String::from("last-epoch"));
				s.insert(String::from("you_tube_3"));
				s.insert(String::from("path-of-exile-2"));
				s.insert(String::from("spongebob"));
				s.insert(String::from("facebook-jfdksalfhdka"));
				s.insert(String::from("facebook-djrieoruaiea"));
				s.insert(String::from("facebook-fjdasklfjsad"));			
			}),
			// ..previous_state
			..CurrentState::load()
	}
		.store();

	println!("Done");
}