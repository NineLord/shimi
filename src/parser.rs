use clap::Parser;

/// A Script of common things a developer might need.
/// It contains commands that are too incontinent to type every time,
/// or just hard to remember.
#[derive(Parser, Debug)]
#[command(name = "s", bin_name = "s", version, about, long_about = None, verbatim_doc_comment)]
pub struct Categories {
    /// Prints verbosely about what's going on.
    #[arg(short = 'd', long = "debug")]
    is_debug: bool,
}