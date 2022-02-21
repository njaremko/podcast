use crate::structs::*;
use crate::utils;

use std::collections::HashSet;
use std::io::{self};

use anyhow::Result;
use async_compat::Compat;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::{self, header};
use smol::io::AsyncWriteExt;

/// This handles downloading a single episode
///
/// Not to be used in conjunction with download_multiple_episodes
async fn download_episode(pb: ProgressBar, episode: Download) -> Result<()> {
    let title = truncate_title(&episode.title);
    pb.set_message(title);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{eta_precise}] {msg} [{bytes_per_sec}] [{bytes}/{total_bytes}]"),
    );
    let client = reqwest::Client::new();
    let mut request = client.get(&episode.url);
    if episode.path.exists() {
        let size = episode.path.metadata()?.len() - 1;
        request = request.header(header::RANGE, format!("bytes={}-", size));
        pb.inc(size);
    }
    let mut dest = smol::io::BufWriter::new(
        smol::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&episode.path)
            .await?,
    );

    let mut download = request.send().await?;
    while let Some(chunk) = download.chunk().await? {
        let written = dest.write(&chunk).await?;
        pb.inc(written as u64);
        let title = truncate_title(&episode.title);
        pb.set_message(title);
    }
    dest.flush().await?;
    pb.finish_with_message("Done");
    Ok(())
}

fn truncate_title(title: &str) -> String {
    let fix_char_len = 45;
    let mut title = title.to_owned();
    if let Some((w, _)) = term_size::dimensions() {
        if fix_char_len < w {
            let new_width = w - fix_char_len;
            title.truncate(new_width);
        } else {
            title.truncate(10)
        }
    } else {
        title.truncate(40);
    }
    title
}

/// Handles downloading a list of episodes on a single thread
async fn download_multiple_episodes(pb: ProgressBar, episodes: Vec<Download>) -> Result<()> {
    let client = reqwest::Client::new();
    for (index, episode) in episodes.iter().enumerate() {
        let title = truncate_title(&episode.title);

        pb.set_position(0);
        pb.set_length(episode.size);
        pb.set_message(title);
        pb.set_style(ProgressStyle::default_bar().template(
            &(format!("[{}/{}]", index, episodes.len())
                + " [{eta_precise}] {msg} [{bytes_per_sec}] [{bytes}/{total_bytes}]"),
        ));
        let mut request = client.get(&episode.url);
        if episode.path.exists() {
            let size = episode.path.metadata()?.len() - 1;
            request = request.header(header::RANGE, format!("bytes={}-", size));
            pb.inc(size);
        }
        let mut dest = smol::io::BufWriter::new(
            smol::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&episode.path)
                .await?,
        );

        let mut download = request.send().await?;
        while let Some(chunk) = download.chunk().await? {
            let written = dest.write(&chunk).await?;
            pb.inc(written as u64);
            let title = truncate_title(&episode.title);
            pb.set_message(title);
        }
        dest.flush().await?;
    }
    pb.finish_with_message("Done");
    Ok(())
}

/// Splits the given list optimally across available threads and downloads them pretty
pub async fn download_episodes(episodes: Vec<Download>) -> Result<()> {
    if episodes.is_empty() {
        return Ok(());
    }

    let mp = MultiProgress::new();
    let num_cpus = num_cpus::get();
    if episodes.len() < num_cpus {
        for episode in episodes.to_owned() {
            let pb = mp.add(ProgressBar::new(episode.size));
            std::thread::spawn(move || smol::block_on(Compat::new(download_episode(pb, episode))));
        }
        mp.join_and_clear()?;
        return Ok(());
    }

    let chunk_size = episodes.len() / num_cpus;
    for chunk in episodes.chunks(chunk_size) {
        let pb = mp.add(ProgressBar::new(0));
        let cp = chunk.to_vec();
        std::thread::spawn(move || smol::block_on(Compat::new(download_multiple_episodes(pb, cp))));
    }
    mp.join_and_clear()?;
    Ok(())
}

pub async fn download_range(
    state: &State,
    p_search: &str,
    e_search: &str,
) -> Result<Vec<Download>> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;
    let mut downloads = vec![];
    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)?;
            let episodes = podcast.episodes();
            let episodes_to_download = parse_download_episodes(e_search)?;

            for ep_num in episodes_to_download {
                let mut path = utils::get_podcast_dir()?;
                path.push(podcast.title());
                utils::create_dir_if_not_exist(&path)?;
                let episode = &episodes[episodes.len() - ep_num];
                if let Some(ep) = Download::new(&state, &podcast, &episode).await? {
                    downloads.push(ep);
                }
            }
        }
    }
    Ok(downloads)
}

fn find_matching_podcast(state: &State, p_search: &str) -> Result<Option<Podcast>> {
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
) -> Result<Vec<Download>> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;
    let mut downloads = vec![];
    if let Ok(ep_num) = e_search.parse::<usize>() {
        for subscription in &state.subscriptions {
            if re_pod.is_match(&subscription.title) {
                let podcast = Podcast::from_title(&subscription.title)?;
                let episodes = podcast.episodes();
                if let Some(ep) =
                    Download::new(&state, &podcast, &episodes[episodes.len() - ep_num]).await?
                {
                    downloads.push(ep);
                }
            }
        }
    } else {
        eprintln!("Failed to parse episode number...\nAttempting to find episode by name...");
        download_episode_by_name(state, p_search, e_search, false).await?;
    }

    Ok(downloads)
}

pub async fn download_episode_by_name(
    state: &State,
    p_search: &str,
    e_search: &str,
    download_all: bool,
) -> Result<Vec<Download>> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    let mut downloads = vec![];
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
                for episode in filtered_episodes {
                    if let Some(ep) = Download::new(&state, &podcast, &episode).await? {
                        downloads.push(ep);
                    }
                }
            } else {
                for episode in filtered_episodes.take(1) {
                    if let Some(ep) = Download::new(&state, &podcast, &episode).await? {
                        downloads.push(ep);
                    }
                }
            }
        }
    }
    Ok(downloads)
}

pub async fn download_all(state: &State, p_search: &str) -> Result<Vec<Download>> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;
    let mut downloads = vec![];
    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)?;
            print!(
                "You are about to download all episodes of {} (y/n): ",
                podcast.title()
            );
            use std::io::Write;
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.to_lowercase().trim() != "y" {
                continue;
            }

            let mut path = utils::get_podcast_dir()?;
            path.push(podcast.title());

            for downloaded in utils::already_downloaded(podcast.title()) {
                let episodes = podcast.episodes();
                for e in episodes
                    .iter()
                    .filter(|e| e.title().is_some())
                    .filter(|e| !downloaded.contains(&e.title().unwrap()))
                {
                    if let Some(ep) = Download::new(&state, &podcast, &e).await? {
                        downloads.push(ep);
                    }
                }
            }
        }
    }
    Ok(downloads)
}

pub async fn download_latest(
    state: &State,
    p_search: &str,
    latest: usize,
) -> Result<Vec<Download>> {
    let mut downloads = vec![];
    if let Some(podcast) = find_matching_podcast(state, p_search)? {
        let episodes = podcast.episodes();
        for episode in &episodes[..latest] {
            if let Some(ep) = Download::new(&state, &podcast, &episode).await? {
                downloads.push(ep);
            }
        }
    }
    Ok(downloads)
}

pub async fn download_rss(state: &State, url: &str) -> Result<Vec<Download>> {
    let channel = utils::download_rss_feed(url).await?;
    let mut download_limit = state.config.auto_download_limit.unwrap_or(1) as usize;
    let mut downloads = vec![];

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

        for episode in episodes[..download_limit].iter() {
            if let Some(ep) = Download::new(&state, &podcast, &episode).await? {
                downloads.push(ep);
            }
        }
    }
    Ok(downloads)
}

fn parse_download_episodes(e_search: &str) -> Result<HashSet<usize>> {
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
