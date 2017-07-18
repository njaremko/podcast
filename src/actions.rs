use regex::Regex;
use structs::*;
use std::process::{Command, Stdio};
use utils::*;

pub fn list_episodes(state: State, search: &str) {
    let re = Regex::new(&search).unwrap();
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

pub fn list_subscriptions(state: State) {
    for podcast in state.subscriptions() {
        println!("{}", podcast.name);
    }
}

pub fn download_episode(state: State, p_search: &str, e_search: &str) {
    let re_pod = Regex::new(&p_search).unwrap();
    let ep_num = e_search.parse::<usize>().unwrap();

    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.name) {
            let podcast = Podcast::from_url(&subscription.url).unwrap();
            let episodes = podcast.episodes();
            match episodes[episodes.len() - ep_num].download(podcast.title()) {
                Err(err) => println!("{}", err),
                _ => (),
            }
        }
    }
}

pub fn download_all(state: State, p_search: &str) {
    let re_pod = Regex::new(&p_search).unwrap();

    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.name) {
            let podcast = Podcast::from_url(&subscription.url).unwrap();
            podcast.download();
        }
    }
}

pub fn play_episode(state: State, p_search: &str, ep_num_string: &str) {
    let re_pod = Regex::new(&p_search).unwrap();
    let ep_num = ep_num_string.parse::<usize>().unwrap();
    let mut path = get_podcast_dir();
    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.name) {
            let podcast = Podcast::from_url(&subscription.url).unwrap();
            path.push(podcast.title());
            let episodes = podcast.episodes();
            let episode = episodes[episodes.len() - ep_num].clone();

            let mut filename = String::from(episode.title().unwrap());
            filename.push_str(episode.download_extension().unwrap());
            path.push(filename);
            match path.exists() {
                true => launch_mpv(path.to_str().unwrap()),
                false => launch_mpv(episode.url().unwrap()),
            }
        }
    }
}

fn launch_mpv(url: &str) {
    Command::new("mpv")
        .args(&["--audio-display=no", url])
        .status()
        .expect("failed to execute process");
}
