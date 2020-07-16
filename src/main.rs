extern crate chrono;
extern crate clap;
extern crate dirs;
#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;
extern crate rayon;
extern crate regex;
extern crate reqwest;
extern crate rss;
#[macro_use]
extern crate serde_derive;
extern crate percent_encoding;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

mod actions;
mod arg_parser;
mod command_handler;
mod commands;
mod download;
mod migration_handler;
mod parser;
mod playback;
mod structs;
mod utils;

use self::structs::*;
use anyhow::Result;
use std::io::Write;

const VERSION: &str = "0.16.0";

#[tokio::main]
async fn main() -> Result<()> {
    utils::create_directories()?;
    migration_handler::migrate()?;
    let client = reqwest::Client::new();
    let mut state = State::new(&client, VERSION).await?;

    let mut app = parser::get_app(&VERSION);
    let matches = app.clone().get_matches();
    let is_quiet = matches.occurrences_of("quiet") != 0;
    
    let config = Config::new()?;
    if !config.quiet.unwrap_or(false) && !is_quiet {
        let path = utils::get_podcast_dir()?;
        writeln!(std::io::stdout().lock(), "Using PODCAST dir: {:?}", &path).ok();
    }
    
    
    command_handler::handle_matches(&VERSION, &client, &mut state, config, &mut app, &matches).await?;
    state.save()
}
