pub(super) const RETURN: &str = "↩ Return to previous menu";

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
//#endregion

macro_rules! from_repr {
	($enum:ident, $index:ident) => {
		$enum::from_repr(
			u8::try_from($index).expect("Enum not suppose to have more than u8::MAX variants")
		).expect("dialoguer::prompts::select::FuzzySelect insures only valid discriminant will be received")
	};
}
pub(crate) use from_repr;
