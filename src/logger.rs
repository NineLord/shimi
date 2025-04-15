use std::env;
use log::{Level, LevelFilter};
use colog::format::CologStyle;
use colored::Colorize;

pub const ENV_VAR_NAME: &str = "SHIMI_LOG";

pub fn init(is_verbose: bool) {
	let mut builder = env_logger::Builder::new();
	builder.format(colog::formatter(CustomLevelToken::new(is_verbose)));
	if is_verbose {
		builder.filter(None, LevelFilter::Trace);
	} else {
		builder.filter(None, LevelFilter::Info);
	}
    if let Ok(env) = env::var(ENV_VAR_NAME) {
        builder.parse_filters(&env);
    }
	builder.init();
}

struct CustomLevelToken<'a, 'b> {
	is_verbose: bool,
	time_format: &'b [time::format_description::BorrowedFormatItem<'a>],
}
impl CustomLevelToken<'_, '_> {
	pub const fn new(is_verbose: bool) -> Self {
		Self {
			is_verbose,
			time_format: time::macros::format_description!("[day]/[month]/[year] [hour]:[minute]:[second].[subsecond digits:3]")
		}
	}
}
impl CologStyle for CustomLevelToken<'_, '_> {
	fn level_color(&self, level: &Level, msg: &str) -> String {
		match level {
			Level::Error => msg.red(),
			Level::Warn => msg.yellow(),
			Level::Info => msg.white(),
			Level::Debug => msg.green(),
			Level::Trace => msg.magenta(),
		}
		.bold()
		.to_string()
	}

    fn level_token(&self, level: &Level) -> &str {
        match *level {
            Level::Error => "Error",
            Level::Warn => "Warn",
            Level::Info => "Info",
            Level::Debug => "Debug",
            Level::Trace => "Trace",
        }
    }

	fn prefix_token(&self, level: &Level) -> String {
		if self.is_verbose {
			format!(
				"{} {}{}{}\t",
				time::OffsetDateTime::now_local()
					.expect("Failed to get current timestamp")
					.format(&self.time_format)
					.expect("Failed to format current timestamp")
					.white(),
				"[".bright_blue(),
				self.level_color(level, self.level_token(level)),
				"]".bright_blue()
			)
		} else {
			format!(
				"{}{}{}\t",
				"[".bright_blue(),
				self.level_color(level, self.level_token(level)),
				"]".bright_blue()
			)
		}
    }
}
