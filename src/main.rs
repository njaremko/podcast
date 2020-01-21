extern crate chrono;
extern crate clap;
extern crate dirs;
#[allow(unused_imports)]
#[macro_use]
extern crate failure;
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
mod search;
mod structs;
mod utils;

mod errors {
    use failure::Error;
    use std::result;
    pub type Result<T> = result::Result<T, Error>;
}

use self::structs::*;
use errors::Result;

const VERSION: &str = "0.13.0";

#[tokio::main]
async fn main() -> Result<()> {
    utils::create_directories()?;
    migration_handler::migrate()?;
    let mut state = State::new(VERSION).await?;
    let config = Config::new()?;
    let mut app = parser::get_app(&VERSION);
    let matches = app.clone().get_matches();
    command_handler::handle_matches(&VERSION, &mut state, config, &mut app, &matches).await?;
    state.save()
}
