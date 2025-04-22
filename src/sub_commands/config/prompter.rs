use anyhow::Result;
use dialoguer::{FuzzySelect, Select};

const RETURN: &str = "↩ Return to previous menu";

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
pub struct FuzzyItems<'a, T: ToString, F: ToString> {
	prefix: &'a [T],
	dynamic: Option<Vec<F>>,
	is_allow_return: bool,
}

#[derive(Clone, Copy)]
pub enum FuzzySelection {
	Prefix(usize),
	Dynamic(usize),
	Return,
}

impl FuzzySelection {
	fn form_usize<T: ToString, F: ToString>(mut index: usize, items: &FuzzyItems<T, F>) -> Self {
		let FuzzyItems { prefix, dynamic, is_allow_return } = items;

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
	fn to_usize<T: ToString, F: ToString>(self, items: &FuzzyItems<T, F>) -> usize {
		match self {
			Self::Prefix(index) => index,
			Self::Dynamic(index) => {
				debug_assert!(items.dynamic.is_some());
				items.prefix.len() + index
			},
			Self::Return => {
				debug_assert!(items.is_allow_return);
				items.prefix.len() + items.dynamic.as_ref().map_or(0, std::vec::Vec::len)
			}
		}
	}
}

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn fuzzy_select<Prompt: Into<String>, T: ToString, F: ToString>(&self, prompt: Prompt, select: FuzzySelection, items: &FuzzyItems<T, F>) -> Result<FuzzySelection> {
		let FuzzyItems { prefix, dynamic, is_allow_return: _ } = items;
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

		Ok(FuzzySelection::form_usize(fuzzy_select.interact()?, items))
	}
}
//#endregion

//#region Select
pub struct Items<'a, T: ToString> {
	prefix: &'a [T],
	is_allow_return: bool,
}

#[derive(Clone, Copy)]
pub enum Selection {
	Prefix(usize),
	Return,
}

impl Selection {
	fn form_usize<T: ToString>(index: usize, items: &Items<T>) -> Self {
		let Items { prefix, is_allow_return } = items;

		if index < prefix.len() {
			return Self::Prefix(index);
		}

		debug_assert!(is_allow_return);
		debug_assert_eq!(index - prefix.len(), 0);
		Self::Return
	}

	/// # Panics
	/// * If `Self::Dynamic` is chosen but there is no `dynamic` in `Items`.
	fn to_usize<T: ToString>(self, items: &Items<T>) -> usize {
		match self {
			Self::Prefix(index) => index,
			Self::Return => {
				debug_assert!(items.is_allow_return);
				items.prefix.len()
			},
		}
	}
}

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn select<Prompt: Into<String>, T: ToString>(&self, prompt: Prompt, select: Selection, items: &Items<T>) -> Result<Selection> {
		let Items { prefix, is_allow_return: _ } = items;
		let mut select = Select::with_theme(&self.theme)
			.with_prompt(prompt)
			.report(false)
			.default(select.to_usize(items))
			.items(prefix);
		if items.is_allow_return {
			select = select.item(RETURN);
		}

		Ok(Selection::form_usize(select.interact()?, items))
	}
}
//#endregion
