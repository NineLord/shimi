use anyhow::Result;
use dialoguer::Input as PromptInput;
use super::prompter::Prompter;

pub enum Input {
	NoneEmpty(String),
	Empty,
}

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	#[allow(dead_code)]
	pub fn input<Prompt: Into<String>>(&self, prompt: Prompt, initial_text: Option<&str>) -> Result<Input> {
		let mut input = PromptInput::with_theme(&self.theme)
			.with_prompt(prompt)
			.allow_empty(true)
			.report(false);
		if let Some(initial_text) = initial_text {
			input = input.with_initial_text(initial_text);
		}
		let result: String = input.interact_text()?;
		let result = result.trim().to_owned();

		let result = if result.is_empty() {
			Input::Empty
		} else {
			Input::NoneEmpty(result)
		};

		Ok(result)
	}
}
