use dialoguer::{theme::ColorfulTheme, console::Term};
use crate::utils::ExitError;

pub struct Prompter<Theme: dialoguer::theme::Theme> {
	pub(super) theme: Theme
}

//#region Constructor
impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub const fn new(theme: Theme) -> Self {
		Self {
			theme
		}
	}
}

impl Default for Prompter<ColorfulTheme> {
	fn default() -> Self {
		Self::new(ColorfulTheme::default())
	}
}
//#endregion

pub(super) fn interrupted_handle<T>(interact_result: Result<T, dialoguer::Error>, add_line_on_int: bool) -> anyhow::Result<T> {
	if let Err(dialoguer::Error::IO(error)) = &interact_result {
		if error.kind() == std::io::ErrorKind::Interrupted {
			let terminal = Term::stdout();
			if add_line_on_int {
				terminal.write_line("")?;
			}
			terminal.show_cursor()?;
			ExitError::Interrupted.exit();
		}
	}
	Ok(interact_result?)
}
