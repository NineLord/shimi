use anyhow::Result;
use dialoguer::FuzzySelect;

pub struct Prompter<Theme: dialoguer::theme::Theme> {
	theme: Theme
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


//#region FuzzySelect
pub struct Items<'a, T: ToString, F: ToString> {
	prefix: &'a [T],
	dynamic: Option<Vec<F>>,
	is_allow_return: bool,
}

#[derive(Clone, Copy)]
pub enum Selection {
	Prefix(usize),
	Dynamic(usize),
	Return,
}

impl Selection {
	fn form_usize<T: ToString, F: ToString>(mut index: usize, items: &Items<T, F>) -> Self {
		let Items { prefix, dynamic, is_allow_return } = items;

		if index < prefix.len() {
			return Self::Prefix(index);
		}
		index -= prefix.len();

		if let Some(dynamic) = &dynamic {
			if index < dynamic.len() {
				return Self::Dynamic(index);
			}
			index -= dynamic.len();
		}

		debug_assert!(is_allow_return);
		debug_assert_eq!(index, 0);
		Self::Return
	}

	/// # Panics
	/// * If `Self::Dynamic` is chosen but there is no `dynamic` in `Items`.
	fn to_usize<T: ToString, F: ToString>(self, items: &Items<T, F>) -> usize {
		match self {
			Self::Prefix(index) => index,
			Self::Dynamic(index) => {
				debug_assert!(items.dynamic.is_some());
				items.prefix.len() + index
			},
			Self::Return => items.prefix.len() + items.dynamic.as_ref().map_or(0, std::vec::Vec::len)
		}
	}
}

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn fuzzy_select<Prompt: Into<String>, T: ToString, F: ToString>(&self, prompt: Prompt, select: Selection, items: &Items<T, F>) -> Result<Selection> {
		const RETURN: &str = "↩ Return to previous menu";
		let Items { prefix, dynamic, is_allow_return: _ } = items;
		let mut fuzzy_select = FuzzySelect::with_theme(&self.theme)
			.with_prompt(prompt)
			.report(false)
			.default(select.to_usize(items))
			.items(prefix);
		if let Some(dynamic) = dynamic {
			fuzzy_select = fuzzy_select.items(dynamic);
		}
		if items.is_allow_return {
			fuzzy_select = fuzzy_select.item(RETURN);
		}

		Ok(Selection::form_usize(fuzzy_select.interact()?, items))
	}
}
//#endregion
