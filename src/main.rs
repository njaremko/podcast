extern crate chrono;
extern crate clap;
extern crate rayon;
extern crate regex;
extern crate reqwest;
extern crate rss;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate yaml_rust;

mod actions;
mod structs;
mod utils;

use actions::*;
use clap::{Arg, App, SubCommand};
use structs::*;

fn main() {
    let mut state = State::new().expect(
        ".subscription file couldn't be parsed...I probably changed the format...sorry",
    );
    let config = Config::new();
    let matches = App::new("podcast")
        .version("1.0")
        .author("Nathan J. <njaremko@gmail.com>")
        .about("Does awesome things")
        .subcommand(
            SubCommand::with_name("download")
                .about("download episodes of podcast")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1),
                )
                .arg(Arg::with_name("EPISODE").help("Episode index").index(2)),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("list episodes of podcast")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("play")
                .about("list episodes of podcast")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("EPISODE")
                        .help("Episode index")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("search")
                .about("searches for podcasts")
                .arg(Arg::with_name("debug").short("d").help(
                    "print debug information verbosely",
                )),
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
        .subcommand(SubCommand::with_name("update").about(
            "update subscribed podcasts",
        ))
        .get_matches();

    match matches.subcommand_name() {
        Some("download") => {
            let download_matches = matches.subcommand_matches("download").unwrap();
            let podcast = download_matches.value_of("PODCAST").unwrap();
            match download_matches.value_of("EPISODE") {
                Some(ep) => {
                    if String::from(ep).contains(|c| c == '-' || c == ',') {
                        download_range(&state, podcast, ep)
                    } else {
                        download_episode(&state, podcast, ep)
                    }
                }
                None => download_all(&state, podcast),
            }
        }
        Some("list") => {
            let list_matches = matches.subcommand_matches("list").unwrap();
            match list_matches.value_of("PODCAST") {
                Some(regex) => list_episodes(regex),
                None => list_subscriptions(&state),
            }
        }
        Some("play") => {
            let play_matches = matches.subcommand_matches("play").unwrap();
            let podcast = play_matches.value_of("PODCAST").unwrap();
            let episode = play_matches.value_of("EPISODE").unwrap();
            play_episode(&state, podcast, episode);
        }
        Some("subscribe") => {
            state.subscribe(
                matches
                    .subcommand_matches("subscribe")
                    .unwrap()
                    .value_of("URL")
                    .unwrap(),
                &config,
            )
        }
        Some("search") => (),
        Some("update") => update_rss(&mut state),
        _ => (),
    }
    if let Err(err) = state.save() {
        eprintln!("{}", err);
    }
}
