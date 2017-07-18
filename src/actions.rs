use regex::Regex;
use reqwest;
use rss::Channel;
use std::fs::{DirBuilder, File};
use std::io::{Read, Write};
use std::process::Command;
use structs::*;
use utils::*;

pub fn list_episodes(state: &State, search: &str) {
    let re = Regex::new(search).unwrap();
    for podcast in state.subscriptions() {
        if re.is_match(&podcast.name) {
            println!("Episodes for {}:", &podcast.name);
            match Podcast::from_url(&podcast.url) {
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

pub fn update_rss(state: &State) {
    let subs = state.subscriptions();
    for sub in subs {
        let mut path = get_podcast_dir();
        path.push(".rss");
        DirBuilder::new().recursive(true).create(&path).unwrap();
        let channel = Channel::from_url(&sub.url).unwrap();
        let mut filename = String::from(channel.title());
        filename.push_str(".xml");
        path.push(filename);
        let mut file = File::create(&path).unwrap();
        let mut resp = reqwest::get(&sub.url).unwrap();
        let mut content: Vec<u8> = Vec::new();
        resp.read_to_end(&mut content).unwrap();
        file.write_all(&content).unwrap();
    }
}

pub fn list_subscriptions(state: &State) {
    for podcast in state.subscriptions() {
        println!("{}", podcast.name);
    }
}

pub fn download_episode(state: &State, p_search: &str, e_search: &str) {
    let re_pod = Regex::new(p_search).unwrap();
    let ep_num = e_search.parse::<usize>().unwrap();

    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.name) {
            let podcast = Podcast::from_url(&subscription.url).unwrap();
            let episodes = podcast.episodes();
            if let Err(err) = episodes[episodes.len() - ep_num].download(podcast.title()) {
                println!("{}", err);
            }
        }
    }
}

pub fn download_all(state: &State, p_search: &str) {
    let re_pod = Regex::new(p_search).unwrap();

    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.name) {
            let podcast = Podcast::from_url(&subscription.url).unwrap();
            podcast.download();
        }
    }
}

pub fn play_episode(state: &State, p_search: &str, ep_num_string: &str) {
    let re_pod = Regex::new(p_search).unwrap();
    let ep_num = ep_num_string.parse::<usize>().unwrap();
    let mut path = get_podcast_dir();
    path.push(".rss");
    DirBuilder::new().recursive(true).create(&path).unwrap();
    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.name) {
            let mut filename = String::from(subscription.name);
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
                launch_mpv(path.to_str().unwrap());
            } else {
                launch_mpv(episode.url().unwrap());
            }
            return;
        }
    }
}

fn launch_mpv(url: &str) {
    Command::new("mpv")
        .args(&["--audio-display=no", url])
        .status()
        .expect("failed to execute process");
}
