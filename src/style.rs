
// TODO: custom titles in help: https://stackoverflow.com/questions/62249229/rust-clap-custom-headings

// TODO: add this to `top_command.rs`
// #[command(styles=clap_cargo::style::CLAP_STYLING)] // Require: clap-cargo = "0.15.2"
// #[command(styles=get_styles())]

// Require: anstyle = { version = "1.0.11" } # ANSI terminal styles
/*pub fn get_styles() -> clap::builder::Styles { // https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
}*/
