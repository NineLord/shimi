use email_address::EmailAddress;

pub fn is_valid_email(input: &str) -> Result<String, String> {
	if EmailAddress::is_valid(input) {
		Ok(input.to_owned())
	} else {
		Err(format!("Invalid e-mail address"))
	}
}