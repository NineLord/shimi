use clap::CommandFactory;
use clap_complete::{generate_to, shells::Bash};
use std::{env, io::Error};

include!("src/parser.rs");

fn main() -> Result<(), Error> {
    match env::var_os("IS_BUILD_COMPLETION_SCRIPT") {
        Some(value) => {
            if value == "false" {
                return Ok(());
            }
        },
        _ => return Ok(()),
    }
    let output_directory = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(output_directory) => output_directory,
    };

    let path = generate_to(
        Bash,
        &mut Categories::command(), // We need to specify what generator to use
        "shimi_completion",  // We need to specify the bin name manually
        output_directory,   // We need to specify where to write to
    )?;

    println!("cargo:warning=completion file is generated: {path:?}");

    Ok(())
}