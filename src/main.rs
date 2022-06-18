#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

mod actions;
mod cargo_parser;
mod command;
mod download;
mod executor;
mod parser;
mod playback;
mod structs;
mod utils;

use self::structs::*;
use anyhow::Result;
use command::*;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // Create
    utils::create_directories()?;

    // Parse the CLI version from the Cargo.toml file
    let version = cargo_parser::get_cli_version()?;

    // Run CLI parser and get matches
    let app = parser::get_app(&version);
    let matches = app.get_matches();

    // Has the user specified that they want the CLI to do minimal output?
    let is_quiet = matches.occurrences_of("quiet") != 0;

    // Load config file
    let config = Config::load()?.unwrap_or_default();
    if !config.quiet.unwrap_or(false) && !is_quiet {
        let path = utils::get_podcast_dir()?;
        writeln!(std::io::stdout().lock(), "Using PODCAST dir: {:?}", &path).ok();
    }

    // Instantiate the global state of the application
    let state = State::new(&version, config).await?;

    // Parse the state and provided arguments into a command to be run
    let command = parse_command(state, matches);

    // After running the given command, we return a new state to persist
    let new_state = run_command(command).await?;

    // Persist new state
    let public_state: PublicState = new_state.into();
    public_state.save()?;
    Ok(())
}
