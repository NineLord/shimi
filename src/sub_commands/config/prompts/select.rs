use anyhow::Result;
use dialoguer::Select;
use super::{prompter::Prompter, selection::{RETURN, Options, Selection, SelectionReturn, insert_options_and_set_default, AnyItem, Item, Items}};

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn select_with_return<Prompt: Into<String>>(&self, prompt: Prompt, mut options: Options<'_>) -> Result<SelectionReturn> {
		options = options.insert_str(RETURN);

		let selection = self.select(prompt, &options)?;

		let result = if selection.vec_index == options.len() -1 {
			SelectionReturn::Return
		} else {
			SelectionReturn::Selection(selection)
		};

		Ok(result)
	}

	pub fn select<Prompt: Into<String>>(&self, prompt: Prompt, options: &Options<'_>) -> Result<Selection> {
		let mut select = Select::with_theme(&self.theme)
			.with_prompt(prompt)
			.report(false);
		
		insert_options_and_set_default!(select, options);

		Ok(options.get_selection(select.interact()?))
	}
}