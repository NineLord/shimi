use anyhow::Result;
use dialoguer::FuzzySelect;
use super::prompter::{Prompter, RETURN};

pub struct FuzzyItems<'a, T: ToString, F: ToString> {
	prefix: &'a [T],
	dynamic: Option<Vec<F>>,
	is_allow_return: bool,
}

impl <'a, T: ToString> FuzzyItems<'a, T, String> {
	pub const fn new(prefix: &'a [T], is_allow_return: bool) -> Self {
		Self {
			prefix,
			dynamic: None,
			is_allow_return,
		}
	}
}

impl <'a, T: ToString, F: ToString> FuzzyItems<'a, T, F> {
	pub const fn with_dynamic(prefix: &'a [T], dynamic: Vec<F>, is_allow_return: bool) -> Self {
		Self {
			prefix,
			dynamic: Some(dynamic),
			is_allow_return,
		}
	}
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
	/// * If `is_allow_return` is false while the index chose be of Return.
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
