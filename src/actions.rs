use regex::Regex;
use reqwest;
use rayon::prelude::*;
use rss::Channel;
use std::collections::HashSet;
use std::fs::{self, DirBuilder, File};
use std::io::{self, BufReader, Read, Write};
use std::process::Command;
use structs::*;
use utils::*;

pub fn list_episodes(state: &State, search: &str) {
    let re = Regex::new(search).unwrap();
    for podcast in state.subscriptions() {
        if re.is_match(&podcast.title()) {
            println!("Episodes for {}:", &podcast.title());
            match Podcast::from_url(&podcast.url()) {
                Ok(podcast) => {
                    let episodes = podcast.episodes();
                    for (index, episode) in episodes.iter().enumerate() {
                        println!("({}) {}", episodes.len() - index, episode.title().unwrap());
                    }
                }
                Err(err) => println!("{}", err),
            }
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
    state.subs.par_iter_mut().for_each(|mut sub| {
        let mut path = get_podcast_dir();
        path.push(sub.title());
        DirBuilder::new().recursive(true).create(&path).unwrap();

        let mut titles = HashSet::new();
        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            titles.insert(trim_extension(&entry.file_name().into_string().unwrap()));
        }

        let mut resp = reqwest::get(&sub.url()).unwrap();
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
                .for_each(|ref ep| if let Err(err) = ep.download(podcast.title()) {
                    eprintln!("Error downloading {}: {}", podcast.title(), err);
                });
        }
        sub.num_episodes = podcast.episodes().len();
    });
}

pub fn list_subscriptions(state: &State) {
    for podcast in state.subscriptions() {
        println!("{}", podcast.title());
    }
}

pub fn download_range(state: &State, p_search: &str, e_search: &str) {
    let re_pod = Regex::new(p_search).unwrap();

    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.title()) {
            let podcast = Podcast::from_url(&subscription.url()).unwrap();
            match parse_download_episodes(e_search) {
                Ok(episodes_to_download) => podcast.download_specific(episodes_to_download),
                Err(err) => eprintln!("{}", err),
            }
            return;
        }
    }
}

pub fn download_episode(state: &State, p_search: &str, e_search: &str) {
    let re_pod = Regex::new(p_search).unwrap();
    let ep_num = e_search.parse::<usize>().unwrap();

    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.title()) {
            let podcast = Podcast::from_url(&subscription.url()).unwrap();
            let episodes = podcast.episodes();
            if let Err(err) = episodes[episodes.len() - ep_num].download(podcast.title()) {
                eprintln!("{}", err);
            }
        }
    }
}

pub fn download_all(state: &State, p_search: &str) {
    let re_pod = Regex::new(p_search).unwrap();

    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.title()) {
            let podcast = Podcast::from_url(&subscription.url()).unwrap();
            podcast.download();
        }
    }
}

pub fn play_episode(state: &State, p_search: &str, ep_num_string: &str) {
    let re_pod = Regex::new(p_search).unwrap();
    let ep_num = ep_num_string.parse::<usize>().unwrap();
    let mut path = get_podcast_dir();
    path.push(".rss");
    if let Err(err) = DirBuilder::new().recursive(true).create(&path) {
        eprintln!(
            "Couldn't create directory: {}\nReason: {}",
            path.to_str().unwrap(),
            err
        );
        return;
    }
    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.title()) {
            let mut filename = String::from(subscription.title());
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

fn launch_player(url: &str) {
    if let Err(_) = launch_mpv(&url) {
        launch_vlc(url)
    }
}

fn launch_mpv(url: &str) -> Result<(), io::Error> {
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
