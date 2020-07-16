use anyhow::Result;
use crate::structs::*;
use crate::utils::*;

use std::fs::{DirBuilder, File};
use std::io::{self, BufReader, Read, Write};
use std::process::Command;

use regex::Regex;
use rss::Channel;
use std::path::PathBuf;

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
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        match err.kind() {
            io::ErrorKind::NotFound => {
                writeln!(&mut handle, "Couldn't open mpv\nTrying vlc...").ok()
            }
            _ => writeln!(&mut handle, "Error: {}", err).ok(),
        };
    }
    Ok(())
}

fn launch_vlc(url: &str) -> Result<()> {
    if let Err(err) = Command::new("vlc").args(&["-I ncurses", url]).status() {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        match err.kind() {
            io::ErrorKind::NotFound => writeln!(&mut handle, "Couldn't open vlc...aborting").ok(),
            _ => writeln!(&mut handle, "Error: {}", err).ok(),
        };
    }
    Ok(())
}

pub fn play_latest(state: &State, p_search: &str) -> Result<()> {
    let re_pod: Regex = Regex::new(&format!("(?i){}", &p_search))?;
    let mut path: PathBuf = get_xml_dir()?;
    DirBuilder::new().recursive(true).create(&path)?;
    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let mut filename: String = subscription.title.clone();
            filename.push_str(".xml");
            path.push(filename);

            let file: File = File::open(&path)?;
            let podcast: Podcast = Podcast::from(Channel::read_from(BufReader::new(file))?);
            let episodes = podcast.episodes();
            let episode = episodes[0].clone();

            filename = episode.title().unwrap();
            filename.push_str(&episode.extension().unwrap());
            path = get_podcast_dir()?;
            path.push(podcast.title());
            path.push(filename);
            if path.exists() {
                launch_player(path.to_str().unwrap())?;
            } else {
                launch_player(episode.url().unwrap())?;
            }
            return Ok(());
        }
    }
    Ok(())
}

pub fn play_episode_by_num(state: &State, p_search: &str, ep_num_string: &str) -> Result<()> {
    let re_pod: Regex = Regex::new(&format!("(?i){}", &p_search))?;
    if let Ok(ep_num) = ep_num_string.parse::<usize>() {
        let mut path: PathBuf = get_xml_dir()?;
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        if let Err(err) = DirBuilder::new().recursive(true).create(&path) {
            writeln!(
                &mut handle,
                "Couldn't create directory: {}\nReason: {}",
                path.to_str().unwrap(),
                err
            )
            .ok();
            return Ok(());
        }
        for subscription in &state.subscriptions {
            if re_pod.is_match(&subscription.title) {
                let mut filename: String = subscription.title.clone();
                filename.push_str(".xml");
                path.push(filename);

                let file: File = File::open(&path).unwrap();
                let podcast = Podcast::from(Channel::read_from(BufReader::new(file)).unwrap());
                let episodes = podcast.episodes();
                let episode = episodes[episodes.len() - ep_num].clone();

                filename = episode.title().unwrap();
                filename.push_str(&episode.extension().unwrap());
                path = get_podcast_dir()?;
                path.push(podcast.title());
                path.push(filename);
                if path.exists() {
                    launch_player(path.to_str().unwrap())?;
                } else {
                    launch_player(episode.url().unwrap())?;
                }
                return Ok(());
            }
        }
    } else {
        {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            writeln!(&mut handle, "Failed to parse episode index number...").ok();
            writeln!(&mut handle, "Attempting to find episode by name...").ok();
        }
        play_episode_by_name(state, p_search, ep_num_string)?;
    }
    Ok(())
}

pub fn play_episode_by_name(state: &State, p_search: &str, ep_string: &str) -> Result<()> {
    let re_pod: Regex = Regex::new(&format!("(?i){}", &p_search))?;
    let mut path: PathBuf = get_xml_dir()?;
    if let Err(err) = DirBuilder::new().recursive(true).create(&path) {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        writeln!(
            &mut handle,
            "Couldn't create directory: {:?}\nReason: {}",
            path, err
        )
        .ok();
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
            let filtered_episodes: Vec<&Episode> = episodes
                .iter()
                .filter(|ep| {
                    ep.title()
                        .unwrap_or_else(|| "".to_string())
                        .to_lowercase()
                        .contains(&ep_string.to_lowercase())
                })
                .collect();
            if let Some(episode) = filtered_episodes.first() {
                filename = episode.title().unwrap();
                filename.push_str(&episode.extension().unwrap());
                path = get_podcast_dir()?;
                path.push(podcast.title());
                path.push(filename);
                if path.exists() {
                    launch_player(path.to_str().unwrap())?;
                } else {
                    launch_player(episode.url().unwrap())?;
                }
            }
            return Ok(());
        }
    }
    Ok(())
}
