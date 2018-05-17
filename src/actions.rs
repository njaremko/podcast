use structs::*;
use utils::*;

use std::collections::HashSet;
use std::fs::{self, DirBuilder, File};
use std::io::{self, BufReader, Read, Write};
use std::process::Command;

use errors::*;
use rayon::prelude::*;
use regex::Regex;
use reqwest;
use rss::Channel;
use std::path::PathBuf;
use toml;

pub fn list_episodes(search: &str) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let re = Regex::new(&format!("(?i){}", &search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;
    let mut path = get_podcast_dir()?;
    path.push(".rss");
    DirBuilder::new()
        .recursive(true)
        .create(&path)
        .chain_err(|| UNABLE_TO_CREATE_DIRECTORY)?;
    for entry in fs::read_dir(&path).chain_err(|| UNABLE_TO_READ_DIRECTORY)? {
        let entry = entry.chain_err(|| UNABLE_TO_READ_ENTRY)?;
        if re.is_match(&entry.file_name().into_string().unwrap()) {
            let file = File::open(&entry.path()).chain_err(|| UNABLE_TO_OPEN_FILE)?;
            let channel = Channel::read_from(BufReader::new(file))
                .chain_err(|| UNABLE_TO_CREATE_CHANNEL_FROM_FILE)?;
            let podcast = Podcast::from(channel);
            let episodes = podcast.episodes();
            for (num, ep) in episodes.iter().enumerate() {
                write!(
                    &mut handle,
                    "({}) {}\n",
                    episodes.len() - num,
                    ep.title().chain_err(|| "unable to retrieve episode title")?
                ).chain_err(|| "unable to write to stdout")?
            }
            return Ok(());
        }
    }
    Ok(())
}

pub fn subscribe_rss(url: &str) -> Result<Channel> {
    println!("Downloading RSS feed...");
    download_rss_feed(url)
}

pub fn download_rss(config: &Config, url: &str) -> Result<()> {
    println!("Downloading episode(s)...");
    let channel = download_rss_feed(url)?;
    let download_limit = config.auto_download_limit as usize;
    if download_limit > 0 {
        let podcast = Podcast::from(channel);
        let episodes = podcast.episodes();
        episodes[..download_limit].par_iter().for_each(|ep| {
            if let Err(err) = ep.download(podcast.title()) {
                eprintln!("Error downloading {}: {}", podcast.title(), err);
            }
        });
    }
    Ok(())
}

pub fn update_subscription(sub: &mut Subscription) -> Result<()> {
    let mut path: PathBuf = get_podcast_dir()?;
    path.push(&sub.title);
    DirBuilder::new()
        .recursive(true)
        .create(&path)
        .chain_err(|| UNABLE_TO_CREATE_DIRECTORY)?;

    let mut titles = HashSet::new();
    for entry in fs::read_dir(&path).chain_err(|| UNABLE_TO_READ_DIRECTORY)? {
        let unwrapped_entry = &entry.chain_err(|| UNABLE_TO_READ_ENTRY)?;
        titles.insert(trim_extension(&unwrapped_entry
            .file_name()
            .into_string()
            .unwrap()));
    }

    let mut resp = reqwest::get(&sub.url).chain_err(|| UNABLE_TO_GET_HTTP_RESPONSE)?;
    let mut content: Vec<u8> = Vec::new();
    resp.read_to_end(&mut content)
        .chain_err(|| UNABLE_TO_READ_RESPONSE_TO_END)?;
    let podcast = Podcast::from(Channel::read_from(BufReader::new(&content[..]))
        .chain_err(|| UNABLE_TO_CREATE_CHANNEL_FROM_RESPONSE)?);
    path = get_podcast_dir()?;
    path.push(".rss");

    let mut filename = String::from(podcast.title());
    filename.push_str(".xml");
    path.push(&filename);
    let mut file = File::create(&path).unwrap();
    file.write_all(&content).unwrap();

    if podcast.episodes().len() > sub.num_episodes {
        podcast.episodes()[..podcast.episodes().len() - sub.num_episodes]
            .par_iter()
            .for_each(|ep: &Episode| {
                if let Err(err) = ep.download(podcast.title()) {
                    eprintln!("Error downloading {}: {}", podcast.title(), err);
                }
            });
    }
    sub.num_episodes = podcast.episodes().len();
    Ok(())
}

pub fn update_rss(state: &mut State) {
    println!("Checking for new episodes...");
    let _result: Vec<Result<()>> = state
        .subscriptions
        .par_iter_mut()
        .map(|sub: &mut Subscription| update_subscription(sub))
        .collect();
}

pub fn list_subscriptions(state: &State) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for podcast in &state.subscriptions() {
        write!(&mut handle, "{}\n", &podcast.title).chain_err(|| "unable to write to stdout")?;
    }
    Ok(())
}

pub fn download_range(state: &State, p_search: &str, e_search: &str) -> Result<()> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)
                .chain_err(|| UNABLE_TO_RETRIEVE_PODCAST_BY_TITLE)?;
            let episodes_to_download = parse_download_episodes(e_search)
                .chain_err(|| "unable to parse episodes to download")?;
            podcast
                .download_specific(&episodes_to_download)
                .chain_err(|| "unable to download episodes")?;
        }
    }
    Ok(())
}

pub fn download_episode(state: &State, p_search: &str, e_search: &str) -> Result<()> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;
    let ep_num = e_search
        .parse::<usize>()
        .chain_err(|| "unable to parse number")?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)
                .chain_err(|| UNABLE_TO_RETRIEVE_PODCAST_BY_TITLE)?;
            let episodes = podcast.episodes();
            episodes[episodes.len() - ep_num]
                .download(podcast.title())
                .chain_err(|| "unable to download episode")?;
        }
    }
    Ok(())
}

pub fn download_all(state: &State, p_search: &str) -> Result<()> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)
                .chain_err(|| UNABLE_TO_RETRIEVE_PODCAST_BY_TITLE)?;
            podcast
                .download()
                .chain_err(|| "unable to download podcast")?;
        }
    }
    Ok(())
}

pub fn play_latest(state: &State, p_search: &str) -> Result<()> {
    let re_pod: Regex =
        Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;
    let mut path: PathBuf = get_xml_dir()?;
    DirBuilder::new()
        .recursive(true)
        .create(&path)
        .chain_err(|| UNABLE_TO_CREATE_DIRECTORY)?;
    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let mut filename: String = subscription.title.clone();
            filename.push_str(".xml");
            path.push(filename);

            let mut file: File = File::open(&path).chain_err(|| UNABLE_TO_OPEN_FILE)?;
            let mut content: Vec<u8> = Vec::new();
            file.read_to_end(&mut content)
                .chain_err(|| "unable to read file to end")?;

            let podcast: Podcast = Podcast::from(Channel::read_from(content.as_slice())
                .chain_err(|| UNABLE_TO_CREATE_CHANNEL_FROM_FILE)?);
            let episodes = podcast.episodes();
            let episode = episodes[0].clone();

            filename = String::from(episode
                .title()
                .chain_err(|| "unable to retrieve episode name")?);
            filename.push_str(episode
                .extension()
                .chain_err(|| "unable to retrieve episode extension")?);
            path = get_podcast_dir()?;
            path.push(podcast.title());
            path.push(filename);
            if path.exists() {
                launch_player(path.to_str()
                    .chain_err(|| "unable to convert path to &str")?)?;
            } else {
                launch_player(episode
                    .url()
                    .chain_err(|| "unable to retrieve episode url")?)?;
            }
            return Ok(());
        }
    }
    Ok(())
}

pub fn play_episode(state: &State, p_search: &str, ep_num_string: &str) -> Result<()> {
    let re_pod: Regex =
        Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;
    let ep_num: usize = ep_num_string.parse::<usize>().unwrap();
    let mut path: PathBuf = get_xml_dir()?;
    if let Err(err) = DirBuilder::new().recursive(true).create(&path) {
        eprintln!(
            "Couldn't create directory: {}\nReason: {}",
            path.to_str().unwrap(),
            err
        );
        return Ok(());
    }
    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let mut filename: String = subscription.title.clone();
            filename.push_str(".xml");
            path.push(filename);

            let mut file: File = File::open(&path).unwrap();
            let mut content: Vec<u8> = Vec::new();
            file.read_to_end(&mut content).unwrap();

            let podcast = Podcast::from(Channel::read_from(content.as_slice()).unwrap());
            let episodes = podcast.episodes();
            let episode = episodes[episodes.len() - ep_num].clone();

            filename = String::from(episode.title().unwrap());
            filename.push_str(episode.extension().unwrap());
            path = get_podcast_dir()?;
            path.push(podcast.title());
            path.push(filename);
            if path.exists() {
                launch_player(path.to_str().chain_err(|| UNABLE_TO_CONVERT_TO_STR)?)?;
            } else {
                launch_player(episode
                    .url()
                    .chain_err(|| "unable to retrieve episode url")?)?;
            }
            return Ok(());
        }
    }
    Ok(())
}

pub fn check_for_update(version: &str) -> Result<()> {
    println!("Checking for updates...");
    let resp: String = reqwest::get(
        "https://raw.githubusercontent.com/njaremko/podcast/master/Cargo.toml",
    ).chain_err(|| UNABLE_TO_GET_HTTP_RESPONSE)?
        .text()
        .chain_err(|| "unable to convert response to text")?;

    //println!("{}", resp);
    let config = resp.parse::<toml::Value>()
        .chain_err(|| "unable to parse toml")?;
    let latest = config["package"]["version"]
        .as_str()
        .chain_err(|| UNABLE_TO_CONVERT_TO_STR)?;
    if version != latest {
        println!("New version avaliable: {} -> {}", version, latest);
    }
    Ok(())
}

fn launch_player(url: &str) -> Result<()> {
    if launch_mpv(url).is_err() {
        return launch_vlc(url);
    }
    Ok(())
}

fn launch_mpv(url: &str) -> Result<()> {
    if let Err(err) = Command::new("mpv")
        .args(&["--audio-display=no", "--ytdl=no", url])
        .status()
    {
        match err.kind() {
            io::ErrorKind::NotFound => {
                eprintln!("Couldn't open mpv\nTrying vlc...");
            }
            _ => eprintln!("Error: {}", err),
        }
    }
    Ok(())
}

fn launch_vlc(url: &str) -> Result<()> {
    if let Err(err) = Command::new("vlc").args(&["-I ncurses", url]).status() {
        match err.kind() {
            io::ErrorKind::NotFound => {
                eprintln!("Couldn't open vlc...aborting");
            }
            _ => eprintln!("Error: {}", err),
        }
    }
    Ok(())
}

pub fn remove_podcast(state: &mut State, p_search: &str) -> Result<()> {
    if p_search == "*" {
        return Podcast::delete_all();
    }

    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;

    for subscription in 0..state.subscriptions.len() {
        let title = state.subscriptions[subscription].title.clone();
        if re_pod.is_match(&title) {
            state.subscriptions.remove(subscription);
            Podcast::delete(&title)?;
        }
    }
    Ok(())
}

pub fn print_completion(arg: &str) {
    let zsh = r#"#compdef podcast
#autoload

# Copyright (C) 2017:
#    Nathan Jaremko <njaremko@gmail.com>
# All Rights Reserved.
# This file is licensed under the GPLv2+. Please see COPYING for more information.

_podcast() {
    local ret=1
    _arguments -C \
        '1: :_podcast_cmds' \
        && ret=0
}

_podcast_cmds () {
    local subcommands;
    subcommands=(
    "download:Download episodes of podcast"
    "help:Prints this message or the help of the given subcommand(s)"
    "ls:List podcasts or episodes of a podcast"
    "play:Play episodes of a podcast"
    "refresh:Refreshes subscribed podcasts"
    "rm:Unsubscribe from a podcast"
    "completion:Shell Completions"
    "search:Searches for podcasts"
    "subscribe:Subscribe to a podcast RSS feed"
    "update:check for updates"
    )
    _describe -t commands 'podcast' subcommands
    _arguments : \
        "--version[Output version information]" \
        "--help[Output help message]"
}

_podcast"#;

    //let bash = "";
    //let sh = "";
    match arg {
        "zsh" => println!("{}", zsh),
        //"bash" => println!("{}", bash),
        //"sh" => println!("{}", sh),
        _ => println!("Only options avaliable are: zsh"),
    }
}
