use clap::{App, ArgMatches};

use std::env;
use std::io;
use std::io::Write;
use std::path::Path;

use crate::actions::*;
use crate::download;
use crate::errors::*;
use crate::playback;
use crate::search;
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

pub fn subscribe(state: &mut State, config: Config, matches: &ArgMatches) -> Result<()> {
    let subscribe_matches = matches
        .subcommand_matches("sub")
        .or_else(|| matches.subcommand_matches("subscribe"))
        .unwrap();
    let url = subscribe_matches.value_of("URL").unwrap();
    sub(state, config, url)?;
    Ok(())
}

fn sub(state: &mut State, config: Config, url: &str) -> Result<()> {
    state.subscribe(url)?;
    download::download_rss(config, url)?;
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

pub fn search(state: &mut State, config: Config, matches: &ArgMatches) -> Result<()> {
    let matches = matches.subcommand_matches("search").unwrap();
    let podcast = matches.value_of("PODCAST").unwrap();
    let resp = search::search_for_podcast(podcast)?;
    if resp.found().is_empty() {
        println!("No Results");
        return Ok(());
    }

    {
        let stdout = io::stdout();
        let mut lock = stdout.lock();
        for (i, r) in resp.found().iter().enumerate() {
            writeln!(&mut lock, "({}) {}", i, r)?;
        }
    }

    print!("Would you like to subscribe to any of these? (y/n): ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.to_lowercase().trim() != "y" {
        return Ok(());
    }

    print!("Which one? (#): ");
    io::stdout().flush().ok();
    let mut num_input = String::new();
    io::stdin().read_line(&mut num_input)?;
    let n: usize = num_input.trim().parse()?;
    if n > resp.found().len() {
        eprintln!("Invalid!");
        return Ok(());
    }

    let rss_resp = search::retrieve_rss(&resp.found()[n])?;
    if let Some(err) = rss_resp.error() {
        eprintln!("{}", err);
        return Ok(());
    }
    match rss_resp.url() {
        Some(r) => sub(state, config, r)?,
        None => eprintln!("Subscription failed. No url in API response."),
    }
    
    Ok(())
}
