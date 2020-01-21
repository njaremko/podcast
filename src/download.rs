use crate::structs::*;
use crate::utils;

use failure::Fail;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};

use failure::Error;
use regex::Regex;
use reqwest;

pub async fn download_range(state: &State, p_search: &str, e_search: &str) -> Result<(), Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    let mut d_vec = vec![];
    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)?;
            let episodes = podcast.episodes();
            let episodes_to_download = parse_download_episodes(e_search)?;

            for ep_num in episodes_to_download {
                let d = download(podcast.title().into(), episodes[episodes.len() - ep_num].clone());
                d_vec.push(d);
            } 
        }
    }
    for c in futures::future::join_all(d_vec).await.iter() {
        if let Err(err) = c {
            println!("Error: {}", err);
        }
    }
    Ok(())
}

fn find_matching_podcast(state: &State, p_search: &str) -> Result<Option<Podcast>, Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;
    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)?;
            return Ok(Some(podcast));
        }
    }
    Ok(None)
}

pub async fn download_episode_by_num(
    state: &State,
    p_search: &str,
    e_search: &str,
) -> Result<(), Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    if let Ok(ep_num) = e_search.parse::<usize>() {
        let mut d_vec = vec![];
        for subscription in &state.subscriptions {
            if re_pod.is_match(&subscription.title) {
                let podcast = Podcast::from_title(&subscription.title)?;
                let episodes = podcast.episodes();
                d_vec.push(download(podcast.title().into(), episodes[episodes.len() - ep_num].clone()));
            }
        }
        for c in futures::future::join_all(d_vec).await.iter() {
            if let Err(err) = c {
                println!("Error: {}", err);
            }
        }
    } else {
        eprintln!("Failed to parse episode number...\nAttempting to find episode by name...");
        download_episode_by_name(state, p_search, e_search, false).await?;
    }

    Ok(())
}

#[derive(Debug, Fail)]
enum DownloadError {
    #[fail(display = "File already exists: {}", path)]
    AlreadyExists { path: String },
}

pub async fn download(podcast_name: String, episode: Episode) -> Result<(), Error> {
    let mut path = utils::get_podcast_dir()?;
    path.push(podcast_name);
    utils::create_dir_if_not_exist(&path)?;

    if let (Some(mut title), Some(url)) = (episode.title(), episode.url()) {
        if let Some(ext) = episode.extension() {
            title = utils::append_extension(&title, &ext);
        }
        path.push(title);
        if !path.exists() {
            println!("Downloading: {:?}", &path);
            let resp = reqwest::get(url).await?.bytes().await?;
            let file = File::create(&path)?;
            let mut reader = BufReader::new(&resp[..]);
            let mut writer = BufWriter::new(file);
            io::copy(&mut reader, &mut writer)?;
        } else {
            return Err(DownloadError::AlreadyExists {
                path: path.to_str().unwrap().to_string(),
            }
            .into());
        }
    }
    Ok(())
}

pub async fn download_episode_by_name(
    state: &State,
    p_search: &str,
    e_search: &str,
    download_all: bool,
) -> Result<(), Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    let mut d_vec = vec![];
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
                for ep in filtered_episodes {
                    let d = download(podcast.title().into(), ep.clone());
                    d_vec.push(d);
                }
            } else {
                for ep in filtered_episodes.take(1) {
                    let d = download(podcast.title().into(), ep.clone());
                    d_vec.push(d);
                }
            }
        }  
    }
    for c in futures::future::join_all(d_vec).await.iter() {
        if let Err(err) = c {
            println!("Error: {}", err);
        }
    }
    Ok(())
}

pub async fn download_all(state: &State, p_search: &str) -> Result<(), Error> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    let mut d_vec = vec![];
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

            let mut path = utils::get_podcast_dir()?;
            path.push(podcast.title());

            for downloaded in utils::already_downloaded(podcast.title()) {
                let episodes = podcast.episodes();
                for e in episodes
                    .iter()
                    .filter(|e| e.title().is_some())
                    .filter(|e| !downloaded.contains(&e.title().unwrap()))
                    .cloned()
                {
                    let d = download(podcast.title().into(), e);
                    d_vec.push(d);
                }
            }
        }
    }
    for c in futures::future::join_all(d_vec).await.iter() {
        if let Err(err) = c {
            println!("Error: {}", err);
        }
    }
    Ok(())
}

pub async fn download_latest(state: &State, p_search: &str, latest: usize) -> Result<(), Error> {
    if let Some(podcast) = find_matching_podcast(state, p_search)? {
        let episodes = podcast.episodes();
        let mut d_vec = vec![];
        for ep in &episodes[..latest] {
            d_vec.push(download(podcast.title().into(), ep.clone()));
        }
        for c in futures::future::join_all(d_vec).await.iter() {
            if let Err(err) = c {
                println!("Error: {}", err);
            }
        }
    }
    Ok(())
}

pub async fn download_rss(config: Config, url: &str) -> Result<(), Error> {
    let channel = utils::download_rss_feed(url).await?;
    let mut download_limit = config.auto_download_limit.unwrap_or(1) as usize;
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

        let mut d_vec = vec![];
        for ep in episodes[..download_limit].iter() {
            d_vec.push(download(podcast.title().into(), ep.clone()));
        }
        for c in futures::future::join_all(d_vec).await.iter() {
            if let Err(err) = c {
                eprintln!("Error downloading {}: {}", podcast.title(), err)
            }
        }
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
                .map(str::parse)
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
