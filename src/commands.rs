use anyhow::Result;
use super::GlobalOptions;

pub trait Run : Sized {
	/// # Errors
	/// Should return an error with explanation why the command couldn't run.
	fn run(self, global_options: &GlobalOptions) -> Result<()>;
}

pub trait GetSubCommandAliases {
	fn get_sub_command_aliases() -> &'static [&'static str];
}

pub trait GetSubCommandNames {
	fn get_sub_command_names(self) -> impl Iterator<Item = &'static str>;
}

pub trait GetSubCommandsNames {
	fn get_sub_commands_names() -> impl Iterator<Item = &'static str>;
}


pub struct ExpandedAlias {
	pub(crate) command: &'static str,
	pub(crate) sub_command: &'static str,
}

pub trait GetExpandedAliases {
	fn get_expanded_aliases(self) -> impl Iterator<Item = ExpandedAlias>;
}

pub trait GetAllExpandedAliases {
	fn get_all_expanded_aliases() -> impl Iterator<Item = ExpandedAlias>;
}
