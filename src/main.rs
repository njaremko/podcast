#![recursion_limit = "1024"]

extern crate chrono;
extern crate clap;
extern crate dirs;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate rayon;
extern crate regex;
extern crate reqwest;
extern crate rss;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

pub mod actions;
pub mod match_handler;
pub mod migration_handler;
pub mod parser;
pub mod structs;
pub mod utils;
pub mod errors {
    error_chain! {}
}

use self::errors::*;
use self::structs::*;

const VERSION: &str = "0.9.1";

fn main() -> Result<()> {
    utils::create_directories().chain_err(|| "unable to create directories")?;
    migration_handler::migrate_old_subscriptions()?;
    let mut state = State::new(VERSION).chain_err(|| "unable to load state")?;
    let config = Config::new()?;
    let mut app = parser::get_app(&VERSION);
    let matches = app.clone().get_matches();
    match_handler::handle_matches(&VERSION, &mut state, &config, &mut app, &matches)?;
    state.save().chain_err(|| "unable to save state")
}
