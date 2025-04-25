use anyhow::Result;
use dialoguer::FuzzySelect;
use super::{prompter::Prompter, selection::{Options, Selection, SelectionReturn, insert_options_and_set_default, AnyItem, Item, Items, Selected, SelectedReturn}};

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn fuzzy_select_with_return<Prompt: Into<String>>(&self, prompt: Prompt, options: &Options<'_, SelectedReturn>) -> Result<SelectionReturn> {
		let mut fuzzy_select = self.get_fuzzy_select(prompt);

		insert_options_and_set_default!(fuzzy_select, options);

		Ok(options.get_selection(fuzzy_select.interact()?))
	}

	pub fn fuzzy_select<Prompt: Into<String>>(&self, prompt: Prompt, options: &Options<'_, Selected>) -> Result<Selection> {
		let mut fuzzy_select = self.get_fuzzy_select(prompt);

		insert_options_and_set_default!(fuzzy_select, options);

		Ok(options.get_selection(fuzzy_select.interact()?))
	}

	fn get_fuzzy_select<Prompt: Into<String>>(&self, prompt: Prompt) -> FuzzySelect {
		FuzzySelect::with_theme(&self.theme)
			.with_prompt(prompt)
			.report(false)
	}
}
