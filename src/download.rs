use crate::structs::*;
use crate::utils::*;

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use failure::Fail;

use failure::Error;
use rayon::prelude::*;
use regex::Regex;
use reqwest;

pub fn download_range(state: &State, p_search: &str, e_search: &str) -> Result<(), Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)?;
            let episodes = podcast.episodes();
            let episodes_to_download = parse_download_episodes(e_search)?;

            episodes_to_download
                .par_iter()
                .map(|ep_num| &episodes[episodes.len() - ep_num])
                .map(|ep| download(podcast.title(), ep))
                .flat_map(|e| e.err())
                .for_each(|err| println!("Error: {}", err));
        }
    }
    Ok(())
}

pub fn download_episode_by_num(state: &State, p_search: &str, e_search: &str) -> Result<(), Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    if let Ok(ep_num) = e_search.parse::<usize>() {
        for subscription in &state.subscriptions {
            if re_pod.is_match(&subscription.title) {
                let podcast = Podcast::from_title(&subscription.title)?;
                let episodes = podcast.episodes();
                download(podcast.title(), &episodes[episodes.len() - ep_num])?;
            }
        }
    } else {
        eprintln!("Failed to parse episode number...\nAttempting to find episode by name...");
        download_episode_by_name(state, p_search, e_search, false)?;
    }

    Ok(())
}

#[derive(Debug, Fail)]
enum DownloadError {
    #[fail(display = "File already exists: {}", path)]
    AlreadyExists {
        path: String,
    }
}

pub fn download(podcast_name: &str, episode: &Episode) -> Result<(), Error> {
    let mut path = get_podcast_dir()?;
    path.push(podcast_name);
    create_dir_if_not_exist(&path)?;

    if let (Some(mut title), Some(url)) = (episode.title(), episode.url()) {
        episode.extension().map(|ext|  {
            if !title.ends_with(".") {
                title.push_str(".");
            }
            title.push_str(&ext);
        });   
        path.push(title);
        if !path.exists() {
            println!("Downloading: {:?}", &path);
            let resp = reqwest::get(url)?;
            let file = File::create(&path)?;
            let mut reader = BufReader::new(resp);
            let mut writer = BufWriter::new(file);
            io::copy(&mut reader, &mut writer)?;
        } else {
            return Err(DownloadError::AlreadyExists{path: path.to_str().unwrap().to_string()}.into());
        }
    }
    Ok(())
}

pub fn download_episode_by_name(
    state: &State,
    p_search: &str,
    e_search: &str,
    download_all: bool,
) -> Result<(), Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)?;
            let episodes = podcast.episodes();
            let filtered_episodes =
                episodes
                    .iter()
                    .filter(|ep| ep.title().is_some())
                    .filter(|ep| {
                        ep.title()
                            .unwrap()
                            .to_lowercase()
                            .contains(&e_search.to_lowercase())
                    });

            if download_all {
                filtered_episodes
                    .map(|ep| download(podcast.title(), ep))
                    .flat_map(|e| e.err())
                    .for_each(|err| eprintln!("Error: {}", err));
            } else {
                filtered_episodes
                    .take(1)
                    .map(|ep| download(podcast.title(), ep))
                    .flat_map(|e| e.err())
                    .for_each(|err| eprintln!("Error: {}", err));
            }
        }
    }
    Ok(())
}

pub fn download_all(state: &State, p_search: &str) -> Result<(), Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)?;
            print!(
                "You are about to download all episodes of {} (y/n): ",
                podcast.title()
            );
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.to_lowercase().trim() != "y" {
                return Ok(());
            }

            let mut path = get_podcast_dir()?;
            path.push(podcast.title());

            already_downloaded(podcast.title()).map(|downloaded| {
                podcast
                    .episodes()
                    .par_iter()
                    .filter(|e| e.title().is_some())
                    .filter(|e| !downloaded.contains(&e.title().unwrap()))
                    .map(|e| download(podcast.title(), e))
                    .flat_map(|e| e.err())
                    .for_each(|err| eprintln!("Error: {}", err))
            })?;
        }
    }
    Ok(())
}

pub fn download_rss(config: &Config, url: &str) -> Result<(), Error> {
    let channel = download_rss_feed(url)?;
    let mut download_limit = config.auto_download_limit as usize;
    if 0 < download_limit {
        println!(
            "Subscribe auto-download limit set to: {}\nDownloading episode(s)...",
            download_limit
        );
        let podcast = Podcast::from(channel);
        let episodes = podcast.episodes();
        if episodes.len() < download_limit {
            download_limit = episodes.len()
        }

        episodes[..download_limit]
            .par_iter()
            .map(|ep| download(podcast.title(), ep))
            .flat_map(|e| e.err())
            .for_each(|err| eprintln!("Error downloading {}: {}", podcast.title(), err));
    }
    Ok(())
}

fn parse_download_episodes(e_search: &str) -> Result<HashSet<usize>, Error> {
    let input = String::from(e_search);
    let mut ranges = Vec::<(usize, usize)>::new();
    let mut elements = HashSet::<usize>::new();
    let comma_separated: Vec<&str> = input.split(',').collect();
    for elem in comma_separated {
        if elem.contains('-') {
            let range: Vec<usize> = elem
                .split('-')
                .map(|i| i.parse::<usize>())
                .collect::<Result<Vec<usize>, std::num::ParseIntError>>()?;
            ranges.push((range[0], range[1]));
        } else {
            elements.insert(elem.parse::<usize>()?);
        }
    }

    for range in ranges {
        // Include given episode in the download
        for num in range.0..=range.1 {
            elements.insert(num);
        }
    }
    Ok(elements)
}
