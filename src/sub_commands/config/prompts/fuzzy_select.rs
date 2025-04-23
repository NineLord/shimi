use anyhow::Result;
use dialoguer::FuzzySelect;
use super::prompter::{Prompter, RETURN};

pub struct FuzzyItems<'a, T: ToString, F: ToString> {
	dynamic_1: Option<&'a [F]>,
	prefix: &'a [T],
	dynamic_2: Option<Vec<F>>,
	is_allow_return: bool,
}

impl <'a, T: ToString> FuzzyItems<'a, T, String> {
	pub const fn new(prefix: &'a [T], is_allow_return: bool) -> Self {
		Self {
			dynamic_1: None,
			prefix,
			dynamic_2: None,
			is_allow_return,
		}
	}
}

impl <'a, T: ToString, F: ToString> FuzzyItems<'a, T, F> {
	pub const fn with_dynamic_1(dynamic: &'a [F], prefix: &'a [T], is_allow_return: bool) -> Self {
		Self {
			dynamic_1: Some(dynamic),
			prefix,
			dynamic_2: None,
			is_allow_return,
		}
	}

	pub const fn with_dynamic_2(prefix: &'a [T], dynamic: Vec<F>, is_allow_return: bool) -> Self {
		Self {
			dynamic_1: None,
			prefix,
			dynamic_2: Some(dynamic),
			is_allow_return,
		}
	}

	pub const fn with_dynamic_1_and_2(dynamic_1: &'a [F], prefix: &'a [T], dynamic_2: Vec<F>, is_allow_return: bool) -> Self {
		Self {
			dynamic_1: Some(dynamic_1),
			prefix,
			dynamic_2: Some(dynamic_2),
			is_allow_return,
		}
	}
}

#[derive(Clone, Copy)]
pub enum FuzzySelection {
	Dynamic1(usize),
	Prefix(usize),
	Dynamic2(usize),
	Return,
}

impl FuzzySelection {
	fn form_usize<T: ToString, F: ToString>(mut index: usize, items: &FuzzyItems<T, F>) -> Self {
		let FuzzyItems { dynamic_1, prefix, dynamic_2, is_allow_return } = items;

		if let Some(dynamic_1) = &dynamic_1 {
			if index < dynamic_1.len() {
				return Self::Dynamic1(index);
			}
			index -= dynamic_1.len();
		}

		if index < prefix.len() {
			return Self::Prefix(index);
		}
		index -= prefix.len();

		if let Some(dynamic_2) = &dynamic_2 {
			if index < dynamic_2.len() {
				return Self::Dynamic2(index);
			}
			index -= dynamic_2.len();
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
			Self::Dynamic1(index) => {
				debug_assert!(items.dynamic_1.is_some());
				index
			},
			Self::Prefix(index) => items.dynamic_1.as_ref().map_or(0, |x| x.len()) + index,
			Self::Dynamic2(index) => {
				debug_assert!(items.dynamic_2.is_some());
				items.dynamic_1.as_ref().map_or(0, |x| x.len()) + items.prefix.len() + index
			},
			Self::Return => {
				debug_assert!(items.is_allow_return);
				items.dynamic_1.as_ref().map_or(0, |x| x.len()) + items.prefix.len() + items.dynamic_2.as_ref().map_or(0, Vec::len)
			}
		}
	}
}

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn fuzzy_select<Prompt: Into<String>, T: ToString, F: ToString>(&self, prompt: Prompt, select: FuzzySelection, items: &FuzzyItems<T, F>) -> Result<FuzzySelection> {
		let FuzzyItems { dynamic_1, prefix, dynamic_2, is_allow_return: _ } = items;
		let mut fuzzy_select = FuzzySelect::with_theme(&self.theme)
			.with_prompt(prompt)
			.report(false)
			.default(select.to_usize(items));

		if let Some(dynamic_1) = dynamic_1 {
			fuzzy_select = fuzzy_select.items(dynamic_1);
		}

		fuzzy_select = fuzzy_select.items(prefix);

		if let Some(dynamic_2) = dynamic_2 {
			fuzzy_select = fuzzy_select.items(dynamic_2);
		}

		if items.is_allow_return {
			fuzzy_select = fuzzy_select.item(RETURN);
		}

		Ok(FuzzySelection::form_usize(fuzzy_select.interact()?, items))
	}
}
