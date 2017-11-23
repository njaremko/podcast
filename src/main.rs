extern crate chrono;
extern crate clap;
extern crate rayon;
extern crate regex;
extern crate reqwest;
extern crate rss;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate yaml_rust;

mod actions;
mod structs;
mod utils;

use actions::*;
use clap::{App, Arg, SubCommand};
use structs::*;
use utils::*;

fn main() {
    if let Err(err) = create_directories() {
        eprintln!("{}", err);
        return;
    }
    let mut state = match State::new() {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let config = Config::new();
    let matches = App::new("podcast")
        .version("0.4")
        .author("Nathan J. <njaremko@gmail.com>")
        .about("A command line podcast manager")
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
            SubCommand::with_name("ls")
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
        .subcommand(SubCommand::with_name("refresh").about("refresh subscribed podcasts"))
        .subcommand(SubCommand::with_name("update").about("check for updates"))
        .subcommand(SubCommand::with_name("rm").about("delete podcast"))
        .subcommand(SubCommand::with_name("completions").about("install shell completions"))
        .get_matches();

    match matches.subcommand_name() {
        Some("download") => {
            let download_matches = matches.subcommand_matches("download").unwrap();
            let podcast = download_matches.value_of("PODCAST").unwrap();
            match download_matches.value_of("EPISODE") {
                Some(ep) => if String::from(ep).contains(|c| c == '-' || c == ',') {
                    download_range(&state, podcast, ep)
                } else {
                    download_episode(&state, podcast, ep)
                },
                None => download_all(&state, podcast),
            }
        }
        Some("ls") => {
            let list_matches = matches.subcommand_matches("ls").unwrap();
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
        Some("subscribe") => state.subscribe(
            matches
                .subcommand_matches("subscribe")
                .unwrap()
                .value_of("URL")
                .unwrap(),
            &config,
        ),
        Some("search") => (),
        Some("rm") => (),
        Some("completions") => (),
        Some("refresh") => update_rss(&mut state),
        Some("update") => check_for_update(&mut state),
        _ => (),
    }
    if let Err(err) = state.save() {
        eprintln!("{}", err);
    }
}
