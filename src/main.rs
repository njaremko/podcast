#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

mod actions;
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
use futures::future;
use std::io::Write;
use std::thread;

const VERSION: &str = "0.17.2";

fn main() -> Result<()> {
    // Same number of threads as there are CPU cores.
    let num_threads = num_cpus::get().max(1);

    // Run the thread-local and work-stealing executor on a thread pool.
    for _ in 0..num_threads {
        // A pending future is one that simply yields forever.
        thread::spawn(|| smol::run(future::pending::<()>()));
    }

    smol::block_on(async {
        // Create
        utils::create_directories()?;

        // Run CLI parser and get matches
        let app = parser::get_app(&VERSION);
        let matches = app.clone().get_matches();

        // Has the user specified that they want the CLI to do minimal output?
        let is_quiet = matches.occurrences_of("quiet") != 0;

        // Load config file
        let config = Config::load()?.unwrap_or_default();
        if !config.quiet.unwrap_or(false) && !is_quiet {
            let path = utils::get_podcast_dir()?;
            writeln!(std::io::stdout().lock(), "Using PODCAST dir: {:?}", &path).ok();
        }

        // Instantiate the global state of the application
        let state = State::new(VERSION, config).await?;

        // Parse the state and provided arguments into a command to be run
        let command = parse_command(state, app, matches);

        // After running the given command, we return a new state to persist
        let new_state = run_command(command).await?;

        // Persist new state
        let public_state: PublicState = new_state.into();
        public_state.save()?;
        Ok(())
    })
}
