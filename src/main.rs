extern crate chrono;
extern crate clap;
extern crate rayon;
extern crate regex;
extern crate reqwest;
extern crate rss;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate yaml_rust;

mod actions;
mod structs;
mod utils;

use actions::*;
use utils::*;
use structs::*;

use clap::{App, Arg, SubCommand};

const VERSION: &str = "0.5.3";

fn main() {
    if let Err(err) = create_directories() {
        eprintln!("{}", err);
        return;
    }
    let mut state = match State::new(VERSION) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let config = Config::new();
    let matches = App::new("podcast")
        .version(VERSION)
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
                .about("play an episode")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("EPISODE")
                        .help("Episode index")
                        .required(false)
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
                )
                .arg(
                    Arg::with_name("download")
                        .short("d")
                        .long("download")
                        .help("auto download based on config"),
                ),
        )
        .subcommand(SubCommand::with_name("refresh").about("refresh subscribed podcasts"))
        .subcommand(SubCommand::with_name("update").about("check for updates"))
        .subcommand(
            SubCommand::with_name("rm")
                .about("unsubscribe from a podcast")
                .arg(Arg::with_name("PODCAST").help("Podcast to delete").index(1)),
        )
        .subcommand(
            SubCommand::with_name("completion")
                .about("install shell completion")
                .arg(
                    Arg::with_name("SHELL")
                        .help("Shell to print completion for")
                        .index(1),
                ),
        )
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
        Some("ls") | Some("list") => {
            let list_matches = matches
                .subcommand_matches("ls")
                .or(matches.subcommand_matches("list"))
                .unwrap();
            match list_matches.value_of("PODCAST") {
                Some(regex) => list_episodes(regex),
                None => list_subscriptions(&state),
            }
        }
        Some("play") => {
            let play_matches = matches.subcommand_matches("play").unwrap();
            let podcast = play_matches.value_of("PODCAST").unwrap();
            match play_matches.value_of("EPISODE") {
                Some(episode) => play_episode(&state, podcast, episode),
                None => play_latest(&state, podcast),
            }
        }
        Some("subscribe") => {
            let subscribe_matches = matches.subcommand_matches("subscribe").unwrap();
            let url = subscribe_matches.value_of("URL").unwrap();
            state.subscribe(url);
            if subscribe_matches.occurrences_of("download") > 0 {
                download_rss(&config, url);
            } else {
                subscribe_rss(url);
            }
        }
        Some("search") => println!("This feature is coming soon..."),
        Some("rm") => {
            let rm_matches = matches.subcommand_matches("rm").unwrap();
            match rm_matches.value_of("PODCAST") {
                Some(regex) => remove_podcast(&mut state, regex),
                None => println!(),
            }
        }
        Some("completion") => {
            let matches = matches.subcommand_matches("completion").unwrap();
            match matches.value_of("SHELL") {
                Some(shell) => print_completion(shell),
                None => print_completion(""),
            }
        }
        Some("refresh") => update_rss(&mut state),
        Some("update") => check_for_update(VERSION),
        _ => (),
    }
    if let Err(err) = state.save() {
        eprintln!("{}", err);
    }
}
