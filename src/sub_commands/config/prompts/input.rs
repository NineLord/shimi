use anyhow::Result;
use dialoguer::Input;
use super::prompter::Prompter;

pub enum Selection {
	NoneEmpty(String),
	Empty,
}

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	pub fn input<Prompt: Into<String>>(&self, prompt: Prompt, initial_text: Option<&str>) -> Result<Selection> {
		let mut input = Input::with_theme(&self.theme)
			.with_prompt(prompt)
			.allow_empty(true)
			.report(false);
		if let Some(initial_text) = initial_text {
			input = input.with_initial_text(initial_text);
		}
		let result: String = input.interact()?;
		let result = result.trim().to_owned();

		let result = if result.is_empty() {
			Selection::Empty
		} else {
			Selection::NoneEmpty(result)
		};

		Ok(result)
	}
}
