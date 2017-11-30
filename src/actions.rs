use rayon::prelude::*;
use regex::Regex;
use reqwest;
use rss::Channel;
use std::collections::HashSet;
use std::fs::{self, DirBuilder, File};
use std::io::{self, BufReader, Read, Write};
use std::process::Command;
use structs::*;
use utils::*;
use toml;

pub fn list_episodes(search: &str) {
    let re = Regex::new(&format!("(?i){}", &search)).expect("Failed to parse regex");
    let mut path = get_podcast_dir();
    path.push(".rss");
    DirBuilder::new().recursive(true).create(&path).unwrap();
    for entry in fs::read_dir(&path).unwrap() {
        let entry = entry.unwrap();
        if re.is_match(&entry.file_name().into_string().unwrap()) {
            let file = File::open(&entry.path()).unwrap();
            let channel = Channel::read_from(BufReader::new(file)).unwrap();
            let podcast = Podcast::from(channel);
            let episodes = podcast.episodes();
            for (num, ep) in episodes.iter().enumerate() {
                println!("({}) {}", episodes.len() - num, ep.title().unwrap());
            }
            return;
        }
    }
}

pub fn download_rss(url: &str, config: &Config) {
    println!("Downloading RSS feed...");
    let mut path = get_podcast_dir();
    path.push(".rss");
    DirBuilder::new().recursive(true).create(&path).unwrap();
    let mut resp = reqwest::get(url).unwrap();
    let mut content: Vec<u8> = Vec::new();
    resp.read_to_end(&mut content).unwrap();
    let channel = Channel::read_from(BufReader::new(&content[..])).unwrap();
    let mut filename = String::from(channel.title());
    filename.push_str(".xml");
    path.push(filename);
    let mut file = File::create(&path).unwrap();
    file.write_all(&content).unwrap();

    let download_limit = config.auto_download_limit as usize;
    if download_limit > 0 {
        let podcast = Podcast::from(channel);
        let episodes = podcast.episodes();
        &episodes[..download_limit].par_iter().for_each(|ref ep| {
            if let Err(err) = ep.download(podcast.title()) {
                eprintln!("Error downloading {}: {}", podcast.title(), err);
            }
        });
    }
}

pub fn update_rss(state: &mut State) {
    println!("Checking for new episodes...");
    &state.subs.par_iter_mut().for_each(|sub| {
        let mut path = get_podcast_dir();
        path.push(&sub.title);
        DirBuilder::new().recursive(true).create(&path).unwrap();

        let mut titles = HashSet::new();
        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            titles.insert(trim_extension(&entry.file_name().into_string().unwrap()));
        }

        let mut resp = reqwest::get(&sub.url).unwrap();
        let mut content: Vec<u8> = Vec::new();
        resp.read_to_end(&mut content).unwrap();
        let podcast = Podcast::from(Channel::read_from(BufReader::new(&content[..])).unwrap());
        path = get_podcast_dir();
        path.push(".rss");

        let mut filename = String::from(podcast.title());
        filename.push_str(".xml");
        path.push(&filename);
        let mut file = File::create(&path).unwrap();
        file.write_all(&content).unwrap();

        if podcast.episodes().len() > sub.num_episodes {
            &podcast.episodes()[..podcast.episodes().len() - sub.num_episodes]
                .par_iter()
                .for_each(|ref ep| {
                    if let Err(err) = ep.download(podcast.title()) {
                        eprintln!("Error downloading {}: {}", podcast.title(), err);
                    }
                });
        }
        sub.num_episodes = podcast.episodes().len();
    });
}

pub fn list_subscriptions(state: &State) {
    for podcast in &state.subscriptions() {
        println!("{}", &podcast.title);
    }
}

pub fn download_range(state: &State, p_search: &str, e_search: &str) {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).expect("Failed to parse regex");

    for subscription in &state.subs {
        if re_pod.is_match(&subscription.title) {
            match Podcast::from_title(&subscription.title) {
                Ok(podcast) => match parse_download_episodes(e_search) {
                    Ok(episodes_to_download) => {
                        if let Err(err) = podcast.download_specific(episodes_to_download) {
                            eprintln!("Error: {}", err);
                        }
                    }
                    Err(err) => eprintln!("Error: {}", err),
                },
                Err(err) => eprintln!("Error: {}", err),
            }
        }
    }
}

pub fn download_episode(state: &State, p_search: &str, e_search: &str) {
    let re_pod = Regex::new(p_search).unwrap();
    let ep_num = e_search.parse::<usize>().unwrap();

    for subscription in &state.subs {
        if re_pod.is_match(&subscription.title) {
            match Podcast::from_title(&subscription.title) {
                Ok(podcast) => {
                    let episodes = podcast.episodes();
                    if let Err(err) = episodes[episodes.len() - ep_num].download(podcast.title()) {
                        eprintln!("{}", err);
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
            }
        }
    }
}

pub fn download_all(state: &State, p_search: &str) {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).expect("Failed to parse regex");

    for subscription in &state.subs {
        if re_pod.is_match(&subscription.title) {
            match Podcast::from_title(&subscription.title) {
                Ok(podcast) => if let Err(err) = podcast.download() {
                    eprintln!("{}", err);
                },
                Err(err) => eprintln!("Error: {}", err),
            }
        }
    }
}

pub fn play_episode(state: &State, p_search: &str, ep_num_string: &str) {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).expect("Failed to parse regex");
    let ep_num = ep_num_string.parse::<usize>().unwrap();
    let mut path = get_xml_dir();
    if let Err(err) = DirBuilder::new().recursive(true).create(&path) {
        eprintln!(
            "Couldn't create directory: {}\nReason: {}",
            path.to_str().unwrap(),
            err
        );
        return;
    }
    for subscription in &state.subs {
        if re_pod.is_match(&subscription.title) {
            let mut filename = subscription.title.clone();
            filename.push_str(".xml");
            path.push(filename);

            let mut file = File::open(&path).unwrap();
            let mut content: Vec<u8> = Vec::new();
            file.read_to_end(&mut content).unwrap();

            let podcast = Podcast::from(Channel::read_from(content.as_slice()).unwrap());
            let episodes = podcast.episodes();
            let episode = episodes[episodes.len() - ep_num].clone();

            filename = String::from(episode.title().unwrap());
            filename.push_str(episode.extension().unwrap());
            path = get_podcast_dir();
            path.push(podcast.title());
            path.push(filename);
            if path.exists() {
                launch_player(path.to_str().unwrap());
            } else {
                launch_player(episode.url().unwrap());
            }
            return;
        }
    }
}

pub fn check_for_update(version: &str) {
    println!("Checking for updates...");
    let resp: String = reqwest::get(
        "https://raw.githubusercontent.com/njaremko/podcast/master/Cargo.toml",
    ).unwrap()
        .text()
        .unwrap();

    //println!("{}", resp);
    match resp.parse::<toml::Value>() {
        Ok(config) => {
            let latest = config["package"]["version"].as_str().unwrap();
             if version != latest {
                println!("New version avaliable: {}", latest);
             }
            },
        Err(err) => eprintln!("{}", err),
    }
}

fn launch_player(url: &str) {
    if let Err(_) = launch_mpv(&url) {
        launch_vlc(&url)
    }
}

fn launch_mpv(url: &str) -> io::Result<()> {
    if let Err(err) = Command::new("mpv")
        .args(&["--audio-display=no", "--ytdl=no", url])
        .status()
    {
        match err.kind() {
            io::ErrorKind::NotFound => {
                eprintln!("Couldn't open mpv\nTrying vlc...");
                return Err(err);
            }
            _ => eprintln!("Error: {}", err),
        }
    }
    Ok(())
}

fn launch_vlc(url: &str) {
    if let Err(err) = Command::new("vlc").args(&["-I ncurses", url]).status() {
        match err.kind() {
            io::ErrorKind::NotFound => {
                eprintln!("vlc not found in PATH\nAborting...");
            }
            _ => eprintln!("Error: {}", err),
        }
    }
}


pub fn remove_podcast(state: &mut State, p_search: &str) {
    if p_search == "*" {
        match Podcast::delete_all() {
            Ok(_) => println!("Success"),
            Err(err) => eprintln!("Error: {}", err),
        }
        return;
    }

    let re_pod = Regex::new(&format!("(?i){}", &p_search)).expect("Failed to parse regex");

    for subscription in 0..state.subs.len() {
        let title = state.subs[subscription].title.clone();
        if re_pod.is_match(&title) {
            state.subs.remove(subscription);
            match Podcast::delete(&title) {
                Ok(_) => println!("Success"),
                Err(err) => eprintln!("Error: {}", err),
            }
            break;
        }
    }
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
