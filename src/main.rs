extern crate rss;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate clap;

mod actions;
mod structs;
mod utils;

use actions::*;
use structs::*;
use clap::{Arg, App, SubCommand};

fn main() {
    let mut state = State::new();

    let matches = App::new("podcast")
        .version("1.0")
        .author("Nathan J. <njaremko@gmail.com>")
        .about("Does awesome things")
        .subcommand(
            SubCommand::with_name("list")
                .about("list episodes of podcast")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        //.required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("search")
                .about("searches for podcasts")
                .arg(
                    Arg::with_name("debug")
                        .short("d")
                        .help("print debug information verbosely"),
                ),
        )
        .subcommand(
            SubCommand::with_name("subscribe")
                .about("subscribes to a podcast RSS feed")
                .arg(
                    Arg::with_name("URL")
                        .help("URL to RSS feed")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("update").about("update subscribed podcasts"),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("list") => {
            let list_matches = matches.subcommand_matches("list").unwrap();
            match list_matches.value_of("PODCAST") {
                Some(regex) => list_episodes(state, regex),
                None => list_subscriptions(state),
            }
        }
        Some("subscribe") => {
            state.subscribe(
                matches
                    .subcommand_matches("subscribe")
                    .unwrap()
                    .value_of("URL")
                    .unwrap(),
            )
        }
        Some("search") => (),
        Some("update") => (),
        _ => (),
    }
}
