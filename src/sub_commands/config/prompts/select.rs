use anyhow::Result;
use dialoguer::Select;
use super::prompter::{Prompter, RETURN, GetSelection, Item, Items, Options, Selection, SelectionReturn, insert_options_and_set_default};

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn select_with_return<Prompt: Into<String>>(&self, prompt: Prompt, mut options: Vec<Options<'_>>, is_allow_return: bool) -> Result<SelectionReturn> {
		if is_allow_return {
			options.push(Options::Item(Item::Str(RETURN), false));
		}

		let selection = self.select(prompt, &options)?;

		let result = if selection.vec_index == options.len() -1 {
			SelectionReturn::Return
		} else {
			SelectionReturn::Selection(selection)
		};

		Ok(result)
	}

	pub fn select<Prompt: Into<String>>(&self, prompt: Prompt, options: &Vec<Options<'_>>) -> Result<Selection> {
		let mut select = Select::with_theme(&self.theme)
			.with_prompt(prompt)
			.report(false);
		
		insert_options_and_set_default!(select, options);

		Ok(options.get_selection(select.interact()?))
	}
}