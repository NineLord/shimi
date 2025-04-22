use anyhow::Result;
use dialoguer::Select;
use super::prompter::{Prompter, RETURN};

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
	/// * If `is_allow_return` is false while the index chose be of Return.
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