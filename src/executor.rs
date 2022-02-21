use crate::actions::*;
use crate::download;
use crate::playback;
use crate::{structs::*, utils};
use anyhow::Result;
use clap::ArgMatches;
use download::download_episodes;
use indicatif::ProgressBar;
use regex::Regex;
use std::env;
use std::{
    io::{self, Read, Write},
    path::Path,
};

struct DownloadProgress<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

pub async fn download(state: State, matches: &ArgMatches) -> Result<State> {
    let podcast = matches.value_of("PODCAST").unwrap();
    let mut to_download = vec![];
    match matches.value_of("EPISODE") {
        Some(ep) => {
            if String::from(ep).contains(|c| c == '-' || c == ',') {
                to_download.append(&mut download::download_range(&state, podcast, ep).await?);
            } else if matches.occurrences_of("name") > 0 {
                to_download.append(
                    &mut download::download_episode_by_name(
                        &state,
                        podcast,
                        ep,
                        0 < matches.occurrences_of("all"),
                    )
                    .await?,
                );
            } else {
                to_download
                    .append(&mut download::download_episode_by_num(&state, podcast, ep).await?);
            }
        }
        None => match matches.value_of("latest") {
            Some(num_of_latest) => {
                to_download.append(
                    &mut download::download_latest(&state, podcast, num_of_latest.parse()?).await?,
                );
            }
            None => {
                to_download.append(&mut download::download_all(&state, podcast).await?);
            }
        },
    }
    download_episodes(to_download).await?;
    Ok(state)
}

pub fn list(state: State, matches: &ArgMatches) -> Result<State> {
    match matches.value_of("PODCAST") {
        Some(regex) => list_episodes(regex)?,
        None => list_subscriptions(&state)?,
    }
    Ok(state)
}

pub fn play(state: State, matches: &ArgMatches) -> Result<State> {
    let podcast = matches.value_of("PODCAST").unwrap();
    match matches.value_of("EPISODE") {
        Some(episode) => {
            if matches.occurrences_of("name") > 0 {
                playback::play_episode_by_name(&state, podcast, episode)?
            } else {
                playback::play_episode_by_num(&state, podcast, episode)?
            }
        }
        None => playback::play_latest(&state, podcast)?,
    }
    Ok(state)
}

pub async fn subscribe(state: State, matches: &ArgMatches) -> Result<State> {
    let url = matches.value_of("URL").unwrap();
    sub(state, url).await
}

async fn sub(mut state: State, url: &str) -> Result<State> {
    state.subscribe(url).await?;
    Ok(state)
}

pub fn remove(mut state: State, matches: &ArgMatches) -> Result<State> {
    let p_search = matches.value_of("PODCAST").unwrap();
    if p_search == "*" {
        state.subscriptions = vec![];
        utils::delete_all()?;
        return Ok(state);
    }

    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    if let Some(index) = state
        .subscriptions
        .iter()
        .position(|sub| re_pod.is_match(sub.title()))
    {
        let title = state.subscriptions[index].title().to_owned();
        state.subscriptions.remove(index);
        utils::delete(&title)?;
    }

    Ok(state)
}

pub fn complete(state: State, matches: &ArgMatches) -> Result<State> {
    match matches.value_of("SHELL") {
        Some(shell) => print_completion(&state, shell),
        None => {
            let shell_path_env = env::var("SHELL");
            if let Ok(p) = shell_path_env {
                let shell_path = Path::new(&p);
                if let Some(shell) = shell_path.file_name() {
                    print_completion(&state, shell.to_str().unwrap())
                }
            }
        }
    }
    Ok(state)
}

pub async fn search(state: State, matches: &ArgMatches) -> Result<State> {
    let podcast = matches
        .values_of("PODCAST")
        .unwrap()
        .fold("".to_string(), |acc, x| {
            if acc.is_empty() {
                return acc + x;
            }
            acc + " " + x
        });

    let resp = podcast_search::search(&podcast).await?;
    if resp.results.is_empty() {
        println!("No Results");
        return Ok(state);
    }

    {
        let stdout = io::stdout();
        let mut lock = stdout.lock();
        for (i, r) in resp.results.iter().enumerate() {
            writeln!(
                &mut lock,
                "({}) {} [{}]",
                i,
                r.collection_name.clone().unwrap_or_else(|| "".to_string()),
                r.feed_url.clone().unwrap_or_else(|| "".to_string())
            )?;
        }
    }

    print!("Would you like to subscribe to any of these? (y/n): ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.to_lowercase().trim() != "y" {
        return Ok(state);
    }

    print!("Which one? (#): ");
    io::stdout().flush().ok();
    let mut num_input = String::new();
    io::stdin().read_line(&mut num_input)?;
    let n: usize = num_input.trim().parse()?;
    if n > resp.results.len() {
        eprintln!("Invalid!");
        return Ok(state);
    }

    let rss_resp = &resp.results[n];
    match &rss_resp.feed_url {
        Some(r) => sub(state, &r).await,
        None => {
            eprintln!("Subscription failed. No url in API response.");
            return Ok(state);
        }
    }
}
