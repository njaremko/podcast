use clap::{App, ArgMatches};

use std::env;
use std::path::Path;

use crate::actions::*;
use crate::errors::*;
use crate::structs::*;

pub fn handle_matches(
    version: &str,
    state: &mut State,
    config: &Config,
    app: &mut App,
    matches: &ArgMatches,
) -> Result<()> {
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
        Some("sub") | Some("subscribe") => {
            let subscribe_matches = matches
                .subcommand_matches("sub")
                .or_else(|| matches.subcommand_matches("subscribe"))
                .chain_err(|| "unable to find subcommand matches")?;
            let url = subscribe_matches
                .value_of("URL")
                .chain_err(|| "unable to find subcommand match")?;
            state.subscribe(url).chain_err(|| "unable to subscribe")?;
            download_rss(&config, url)?;
        }
        Some("search") => println!("This feature is coming soon..."),
        Some("rm") => {
            let rm_matches = matches
                .subcommand_matches("rm")
                .chain_err(|| "unable to find subcommand matches")?;
            let regex = rm_matches.value_of("PODCAST").chain_err(|| "")?;
            remove_podcast(state, regex)?
        }
        Some("completion") => {
            let matches = matches
                .subcommand_matches("completion")
                .chain_err(|| "unable to find subcommand matches")?;
            match matches.value_of("SHELL") {
                Some(shell) => print_completion(app, shell),
                None => {
                    let shell_path_env = env::var("SHELL");
                    if let Ok(p) = shell_path_env {
                        let shell_path = Path::new(&p);
                        if let Some(shell) = shell_path.file_name() {
                            print_completion(app, shell.to_str().chain_err(|| format!("Unable to convert {:?} to string", shell))?)
                        }
                    }
                }
            }
        }
        Some("refresh") => update_rss(state),
        Some("update") => check_for_update(version)?,
        _ => (),
    };
    Ok(())
}
