use clap::{App, ArgMatches};

use std::env;
use std::path::Path;

use crate::actions::*;
use crate::download;
use crate::errors::*;
use crate::playback;
use crate::structs::*;

pub fn download(state: &mut State, matches: &ArgMatches) -> Result<()> {
    let download_matches = matches.subcommand_matches("download").unwrap();
    let podcast = download_matches.value_of("PODCAST").unwrap();
    match download_matches.value_of("EPISODE") {
        Some(ep) => {
            if String::from(ep).contains(|c| c == '-' || c == ',') {
                download::download_range(&state, podcast, ep)?
            } else if download_matches.occurrences_of("name") > 0 {
                download::download_episode_by_name(
                    &state,
                    podcast,
                    ep,
                    download_matches.occurrences_of("all") > 0,
                )?
            } else {
                download::download_episode_by_num(&state, podcast, ep)?
            }
        }
        None => download::download_all(&state, podcast)?,
    }
    Ok(())
}

pub fn list(state: &mut State, matches: &ArgMatches) -> Result<()> {
    let list_matches = matches
        .subcommand_matches("ls")
        .or_else(|| matches.subcommand_matches("list"))
        .unwrap();
    match list_matches.value_of("PODCAST") {
        Some(regex) => list_episodes(regex)?,
        None => list_subscriptions(&state)?,
    }
    Ok(())
}

pub fn play(state: &mut State, matches: &ArgMatches) -> Result<()> {
    let play_matches = matches.subcommand_matches("play").unwrap();
    let podcast = play_matches.value_of("PODCAST").unwrap();
    match play_matches.value_of("EPISODE") {
        Some(episode) => {
            if play_matches.occurrences_of("name") > 0 {
                playback::play_episode_by_name(&state, podcast, episode)?
            } else {
                playback::play_episode_by_num(&state, podcast, episode)?
            }
        }
        None => playback::play_latest(&state, podcast)?,
    }
    Ok(())
}

pub fn subscribe(state: &mut State, config: &Config, matches: &ArgMatches) -> Result<()> {
    let subscribe_matches = matches
        .subcommand_matches("sub")
        .or_else(|| matches.subcommand_matches("subscribe"))
        .unwrap();
    let url = subscribe_matches.value_of("URL").unwrap();
    state.subscribe(url)?;
    download::download_rss(&config, url)?;
    Ok(())
}

pub fn remove(state: &mut State, matches: &ArgMatches) -> Result<()> {
    let rm_matches = matches.subcommand_matches("rm").unwrap();
    let regex = rm_matches.value_of("PODCAST").unwrap();
    remove_podcast(state, regex)?;
    Ok(())
}

pub fn complete(app: &mut App, matches: &ArgMatches) -> Result<()> {
    let matches = matches.subcommand_matches("completion").unwrap();
    match matches.value_of("SHELL") {
        Some(shell) => print_completion(app, shell),
        None => {
            let shell_path_env = env::var("SHELL");
            if let Ok(p) = shell_path_env {
                let shell_path = Path::new(&p);
                if let Some(shell) = shell_path.file_name() {
                    print_completion(app, shell.to_str().unwrap())
                }
            }
        }
    }
    Ok(())
}
