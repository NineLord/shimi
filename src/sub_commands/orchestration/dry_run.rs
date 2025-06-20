#![allow(dead_code)]
use crate::prelude::PACKAGE_NAME;
use docker_compose_types::{Command, Service, Services};
use rand::RngCore;
pub use docker_compose_types::Compose as DockerComposeType;

const CONFIG_FILE_NAME: &str = "current_state_dryrun";

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
		Self::load().services.get_container_names()
	}
}

pub struct Stateful {
	pub scale: i64,
	pub naming_convention: NamingConvention,
}

pub enum NamingConvention {
	/// `<service_name>_0`, `<service_name>_1`, ...
	UnderscoreNumber0,
	/// `<service_name>_1`, `<service_name>_2`, ...
	UnderscoreNumber1,
	/// `<service_name>-hsahdsj`, `<service_name>-hdsjakh`, ...
	DashHash,
}

pub enum Scale {
	Stateless(i64),
	Stateful(Stateful),
}

pub trait AddService {
	fn add_service(&mut self, service_name: String, scale: Option<Scale>);
}

impl AddService for Services {
	fn add_service(&mut self, service_name: String, scale: Option<Scale>) {
		let Self(services) = self;

		let mut service = Service {
			image: Some(String::from("alpine:latest")),
			command: Some(Command::Args(vec![
				String::from("sh"),
				String::from("-c"),
				format!("echo {service_name} && tail -f /dev/null")
			])),
			..Default::default()
		};

		let duplicate = match scale {
			Some(Scale::Stateless(scale)) => {
				service.scale = scale;
				None
			},
			Some(Scale::Stateful(scale)) => Some(scale),
			None => {
				service.container_name = Some(service_name.clone());
				None
			},
		};

		match duplicate {
			Some(Stateful { scale, naming_convention: NamingConvention::UnderscoreNumber0 }) => {
				for index in 0..scale {
					let mut service = service.clone();
					let service_name = format!("{service_name}_{index}");
					service.container_name = Some(service_name.clone());
					services.insert(service_name, Some(service));
				}
			},
			Some(Stateful { scale, naming_convention: NamingConvention::UnderscoreNumber1 }) => {
				for index in 1..=scale {
					let mut service = service.clone();
					let service_name = format!("{service_name}_{index}");
					service.container_name = Some(service_name.clone());
					services.insert(service_name, Some(service));
				}
			},
			Some(Stateful { scale, naming_convention: NamingConvention::DashHash }) => {
				let mut rng = rand::rng();
				for _ in 0..scale {
					let mut service = service.clone();
					// Generate a random 6-byte (12 hex chars) hash
					let mut bytes = [0u8; 6];
					rng.fill_bytes(&mut bytes);
					let hash = hex::encode(bytes);
					let service_name = format!("{service_name}-{hash}");
					service.container_name = Some(service_name.clone());
					services.insert(service_name, Some(service));
				}
			},
			None => {
				services.insert(service_name, Some(service));
			},
		}
	}
}

pub trait GetContainerName {
	fn get_container_names(&self) -> Vec<String>;
}

impl GetContainerName for Services {
	fn get_container_names(&self) -> Vec<String> {
		let Self(services) = self;
		services.iter()
			.fold(Vec::with_capacity(services.len()), |mut accumulator, (service_name, service)| {
				match service {
					Some(service) => {
						if service.scale == 0 {
							if let Some(container_name) = &service.container_name {
								accumulator.push(container_name.clone());
							} else {
								accumulator.push(service_name.clone());
							}
						} else {
							for index in 1..=service.scale {
								accumulator.push(format!("junk-{service_name}-{index}"));
							}
						}
					},
					None => accumulator.push(service_name.clone()),
				}
				accumulator
			})
	}
}
