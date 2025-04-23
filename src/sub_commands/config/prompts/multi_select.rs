use anyhow::Result;
use dialoguer::MultiSelect;
use super::prompter::Prompter;

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn multi_select<Prompt: Into<String>, T: ToString>(&self, prompt: Prompt, items: &[T], selected: Option<&[bool]>) -> Result<Option<Vec<usize>>> {
		let mut multi_select = MultiSelect::with_theme(&self.theme)
			.with_prompt(prompt)
			.items(items)
			.report(false);
		if let Some(selected) = selected {
			multi_select = multi_select.defaults(selected);
		}
		Ok(multi_select.interact_opt()?)
	}
}
