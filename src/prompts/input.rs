use anyhow::Result;
use dialoguer::{Input as PromptInput, InputValidator};
use super::prompter::{Prompter, interrupted_handle};

pub enum Input {
	NoneEmpty(String),
	Empty,
}

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	/// # Errors
	/// If the terminal was interrupted
	#[allow(dead_code)]
	pub fn input<Prompt: Into<String>>(&self, prompt: Prompt, initial_text: Option<&str>) -> Result<Input> {
		let none = None as Option<Box<dyn FnMut(&String) -> Result<(), &'static str>>>;
		self.input_generic(prompt, initial_text, none)
	}

	/// # Errors
	/// If the terminal was interrupted
	#[allow(dead_code)]
	pub fn input_with_validation<'a, Prompt, V>(&'a self, prompt: Prompt, initial_text: Option<&str>, validator: V) -> Result<Input>
	where
		Prompt: Into<String>,
		V: InputValidator<String> + 'a,
        V::Err: ToString,
	{
		self.input_generic(prompt, initial_text, Some(validator))
	}

	fn input_generic<'a, Prompt, V>(&'a self, prompt: Prompt, initial_text: Option<&str>, validator: Option<V>) -> Result<Input>
	where
		Prompt: Into<String>,
		V: InputValidator<String> + 'a,
        V::Err: ToString,
	{
		let mut input: PromptInput<'a, String> = PromptInput::with_theme(&self.theme)
			.with_prompt(prompt)
			.allow_empty(true)
			.report(false);
		if let Some(validator) = validator {
			input = input.validate_with(validator);
		}
		if let Some(initial_text) = initial_text {
			input = input.with_initial_text(initial_text);
		}
		let result: String = interrupted_handle(input.interact_text(), true)?;
		let result = result.trim().to_owned();

		let result = if result.is_empty() {
			Input::Empty
		} else {
			Input::NoneEmpty(result)
		};

		Ok(result)
	}
}
