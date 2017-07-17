use regex::Regex;
use structs::*;
use std::process::{Command, Stdio};

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

pub fn stream_episode(state: State, p_search: &str, e_search: &str) {
    let re_pod = Regex::new(&p_search).unwrap();
    let ep_num = e_search.parse::<usize>().unwrap();

    for subscription in state.subscriptions() {
        if re_pod.is_match(&subscription.name) {
            let podcast = Podcast::from_url(&subscription.url).unwrap();
            let episodes = podcast.episodes();
            launch_mpv(episodes[episodes.len() - ep_num].download_url().unwrap());
        }
    }
}

fn launch_mpv(url: &str) {
    Command::new("mpv")
        .args(&["--audio-display=no", url])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");
}
