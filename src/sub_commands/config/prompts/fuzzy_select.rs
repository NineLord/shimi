use anyhow::Result;
use dialoguer::FuzzySelect;
use super::{prompter::Prompter, selection::{RETURN, Options, Selection, SelectionReturn, insert_options_and_set_default, AnyItem, Item, Items}};

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn fuzzy_select_with_return<Prompt: Into<String>>(&self, prompt: Prompt, mut options: Options<'_>, is_allow_return: bool) -> Result<SelectionReturn> {
		if is_allow_return {
			options = options.insert_str(RETURN);
		}

		let selection = self.fuzzy_select(prompt, &options)?;

		let result = if selection.vec_index == options.len() -1 {
			SelectionReturn::Return
		} else {
			SelectionReturn::Selection(selection)
		};

		Ok(result)
	}

	pub fn fuzzy_select<Prompt: Into<String>>(&self, prompt: Prompt, options: &Options<'_>) -> Result<Selection> {
		let mut fuzzy_select = FuzzySelect::with_theme(&self.theme)
			.with_prompt(prompt)
			.report(false);

		insert_options_and_set_default!(fuzzy_select, options);

		Ok(options.get_selection(fuzzy_select.interact()?))
	}
}
