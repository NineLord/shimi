use dialoguer::theme::ColorfulTheme;

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
