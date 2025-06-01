use anyhow::Result;
use dialoguer::Confirm;
use super::prompter::Prompter;

impl <Theme: dialoguer::theme::Theme> Prompter<Theme> {
	/// # Errors
	/// If the terminal was interrupted
	#[allow(dead_code)]
	pub fn confirm<Prompt: Into<String>>(&self, prompt: Prompt) -> Result<bool> {
		let multi_select = Confirm::with_theme(&self.theme)
			.with_prompt(prompt)
			.wait_for_newline(true)
			.report(false)
			.interact()?;

		Ok(multi_select)
	}
}
