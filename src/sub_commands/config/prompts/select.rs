use anyhow::Result;
use dialoguer::Select;
use super::{prompter::Prompter, selection::{Options, Selection, SelectionReturn, insert_options_and_set_default, AnyItem, Item, Items, Selected, SelectedReturn}};

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn select_with_return<Prompt: Into<String>>(&self, prompt: Prompt, options: &Options<'_, SelectedReturn>) -> Result<SelectionReturn> {
		let mut select = self.get_select(prompt);
		
		insert_options_and_set_default!(select, options);

		Ok(options.get_selection(select.interact()?))
	}

	pub fn select<Prompt: Into<String>>(&self, prompt: Prompt, options: &Options<'_, Selected>) -> Result<Selection> {
		let mut select = self.get_select(prompt);
		
		insert_options_and_set_default!(select, options);

		Ok(options.get_selection(select.interact()?))
	}

	fn get_select<Prompt: Into<String>>(&self, prompt: Prompt) -> Select {
		Select::with_theme(&self.theme)
			.with_prompt(prompt)
			.report(false)
	}
}