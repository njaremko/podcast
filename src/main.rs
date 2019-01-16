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
extern crate toml;
extern crate yaml_rust;

pub mod actions;
pub mod structs;
pub mod utils;
pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

use self::actions::*;
use self::errors::*;
use self::structs::*;
use self::utils::*;

use clap::{App, Arg, SubCommand};

const VERSION: &str = "0.7.5";

fn main() -> Result<()> {
    create_directories().chain_err(|| "unable to create directories")?;
    let mut state = State::new(VERSION).chain_err(|| "unable to load state")?;
    let config = Config::new()?;
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
                .arg(
                    Arg::with_name("EPISODE")
                        .required(false)
                        .help("Episode index")
                        .index(2),
                )
                .arg(
                    Arg::with_name("name")
                        .short("e")
                        .long("episode")
                        .help("Download using episode name instead of number")
                        .required(false),
                )
                .arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("Download all matching episodes")
                        .required(false),
                ),
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
                )
                .arg(
                    Arg::with_name("name")
                        .short("e")
                        .long("episode")
                        .help("Play using episode name instead of number")
                        .required(false),
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
            let download_matches = matches
                .subcommand_matches("download")
                .chain_err(|| "unable to find subcommand matches")?;
            let podcast = download_matches
                .value_of("PODCAST")
                .chain_err(|| "unable to find subcommand match")?;
            match download_matches.value_of("EPISODE") {
                Some(ep) => {
                    if String::from(ep).contains(|c| c == '-' || c == ',') {
                        download_range(&state, podcast, ep)?
                    } else if download_matches.occurrences_of("name") > 0 {
                        download_episode_by_name(
                            &state,
                            podcast,
                            ep,
                            download_matches.occurrences_of("all") > 0,
                        )?
                    } else {
                        download_episode_by_num(&state, podcast, ep)?
                    }
                }
                None => download_all(&state, podcast)?,
            }
        }
        Some("ls") | Some("list") => {
            let list_matches = matches
                .subcommand_matches("ls")
                .or_else(|| matches.subcommand_matches("list"))
                .chain_err(|| "unable to find subcommand matches")?;
            match list_matches.value_of("PODCAST") {
                Some(regex) => list_episodes(regex)?,
                None => list_subscriptions(&state)?,
            }
        }
        Some("play") => {
            let play_matches = matches
                .subcommand_matches("play")
                .chain_err(|| "unable to find subcommand matches")?;
            let podcast = play_matches
                .value_of("PODCAST")
                .chain_err(|| "unable to find subcommand match")?;
            match play_matches.value_of("EPISODE") {
                Some(episode) => {
                    if play_matches.occurrences_of("name") > 0 {
                        play_episode_by_name(&state, podcast, episode)?
                    } else {
                        play_episode_by_num(&state, podcast, episode)?
                    }
                }
                None => play_latest(&state, podcast)?,
            }
        }
        Some("subscribe") => {
            let subscribe_matches = matches
                .subcommand_matches("subscribe")
                .chain_err(|| "unable to find subcommand matches")?;
            let url = subscribe_matches
                .value_of("URL")
                .chain_err(|| "unable to find subcommand match")?;
            state.subscribe(url).chain_err(|| "unable to subscribe")?;
            if subscribe_matches.occurrences_of("download") > 0 {
                download_rss(&config, url)?;
            } else {
                subscribe_rss(url)?;
            }
        }
        Some("search") => println!("This feature is coming soon..."),
        Some("rm") => {
            let rm_matches = matches
                .subcommand_matches("rm")
                .chain_err(|| "unable to find subcommand matches")?;
            let regex = rm_matches.value_of("PODCAST").chain_err(|| "")?;
            remove_podcast(&mut state, regex)?
        }
        Some("completion") => {
            let matches = matches
                .subcommand_matches("completion")
                .chain_err(|| "unable to find subcommand matches")?;
            match matches.value_of("SHELL") {
                Some(shell) => print_completion(shell),
                None => print_completion(""),
            }
        }
        Some("refresh") => update_rss(&mut state),
        Some("update") => check_for_update(VERSION)?,
        _ => (),
    }
    state.save().chain_err(|| "unable to save state")
}
