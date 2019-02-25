use crate::structs::*;
use crate::utils::*;

use std::collections::HashSet;
use std::fs::{self, DirBuilder, File};
use std::io::{self, BufReader, Read, Write};
use std::process::Command;

use crate::errors::*;
use clap::App;
use clap::Shell;
use rayon::prelude::*;
use regex::Regex;
use reqwest;
use rss::Channel;
use std::path::PathBuf;
use toml;

pub fn list_episodes(search: &str) -> Result<()> {
    let re = Regex::new(&format!("(?i){}", &search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;
    let path = get_xml_dir()?;
    create_dir_if_not_exist(&path)?;

    for entry in fs::read_dir(&path).chain_err(|| UNABLE_TO_READ_DIRECTORY)? {
        let entry = entry.chain_err(|| UNABLE_TO_READ_ENTRY)?;
        if re.is_match(&entry.file_name().into_string().unwrap()) {
            let file = File::open(&entry.path()).chain_err(|| UNABLE_TO_OPEN_FILE)?;
            let channel = Channel::read_from(BufReader::new(file))
                .chain_err(|| UNABLE_TO_CREATE_CHANNEL_FROM_FILE)?;
            let podcast = Podcast::from(channel);
            let episodes = podcast.episodes();
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            for (num, ep) in episodes.iter().enumerate() {
                writeln!(
                    &mut handle,
                    "({}) {}",
                    episodes.len() - num,
                    ep.title()
                        .chain_err(|| "unable to retrieve episode title")?
                )
                .ok();
            }
            return Ok(());
        }
    }
    Ok(())
}

pub fn download_rss(config: &Config, url: &str) -> Result<()> {
    let channel = download_rss_feed(url)?;
    let mut download_limit = config.auto_download_limit as usize;
    if 0 < download_limit {
        println!("Downloading episode(s)...");
        let podcast = Podcast::from(channel);
        let episodes = podcast.episodes();
        if episodes.len() < download_limit {
            download_limit = episodes.len()
        }
        episodes[..download_limit].par_iter().for_each(|ep| {
            if let Err(err) = download(podcast.title(), ep) {
                eprintln!("Error downloading {}: {}", podcast.title(), err);
            }
        });
    }
    Ok(())
}

pub fn update_subscription(sub: &mut Subscription) -> Result<()> {
    let mut path: PathBuf = get_podcast_dir()?;
    path.push(&sub.title);
    create_dir_if_not_exist(&path)?;

    let mut titles = HashSet::new();
    for entry in fs::read_dir(&path).chain_err(|| UNABLE_TO_READ_DIRECTORY)? {
        let unwrapped_entry = &entry.chain_err(|| UNABLE_TO_READ_ENTRY)?;
        titles.insert(trim_extension(
            &unwrapped_entry.file_name().into_string().unwrap(),
        ));
    }

    let mut resp = reqwest::get(&sub.url).chain_err(|| UNABLE_TO_GET_HTTP_RESPONSE)?;
    let mut content: Vec<u8> = Vec::new();
    resp.read_to_end(&mut content)
        .chain_err(|| UNABLE_TO_READ_RESPONSE_TO_END)?;
    let podcast = Podcast::from(
        Channel::read_from(BufReader::new(&content[..]))
            .chain_err(|| UNABLE_TO_CREATE_CHANNEL_FROM_RESPONSE)?,
    );

    let mut filename = String::from(podcast.title());
    filename.push_str(".xml");

    let mut podcast_rss_path = get_xml_dir()?;
    podcast_rss_path.push(&filename);

    let mut file = File::create(&podcast_rss_path).unwrap();
    file.write_all(&content).unwrap();

    if sub.num_episodes < podcast.episodes().len() {
        podcast.episodes()[..podcast.episodes().len() - sub.num_episodes]
            .par_iter()
            .for_each(|ep: &Episode| {
                if let Err(err) = download(podcast.title(), ep) {
                    eprintln!("Error downloading {}: {}", podcast.title(), err);
                }
            });
    }
    sub.num_episodes = podcast.episodes().len();
    Ok(())
}

pub fn update_rss(state: &mut State) {
    println!("Checking for new episodes...");
    let _result: Vec<Result<()>> = state
        .subscriptions
        .par_iter_mut()
        .map(|sub: &mut Subscription| update_subscription(sub))
        .collect();
}

pub fn list_subscriptions(state: &State) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for podcast in state.subscriptions() {
        writeln!(&mut handle, "{}", &podcast.title).ok();
    }
    Ok(())
}

pub fn download_range(state: &State, p_search: &str, e_search: &str) -> Result<()> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)
                .chain_err(|| UNABLE_TO_RETRIEVE_PODCAST_BY_TITLE)?;
            let episodes_to_download = parse_download_episodes(e_search)
                .chain_err(|| "unable to parse episodes to download")?;
            podcast
                .download_specific(&episodes_to_download)
                .chain_err(|| "unable to download episodes")?;
        }
    }
    Ok(())
}

pub fn download_episode_by_num(state: &State, p_search: &str, e_search: &str) -> Result<()> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;

    if let Ok(ep_num) = e_search.parse::<usize>() {
        for subscription in &state.subscriptions {
            if re_pod.is_match(&subscription.title) {
                let podcast = Podcast::from_title(&subscription.title)
                    .chain_err(|| UNABLE_TO_RETRIEVE_PODCAST_BY_TITLE)?;
                let episodes = podcast.episodes();
                download(podcast.title(), &episodes[episodes.len() - ep_num])
                    .chain_err(|| "unable to download episode")?;
            }
        }
    } else {
        {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            writeln!(
                &mut handle,
                "Failed to parse episode number...\nAttempting to find episode by name..."
            )
            .ok();
        }
        download_episode_by_name(state, p_search, e_search, false)
            .chain_err(|| "Failed to download episode.")?;
    }

    Ok(())
}

pub fn download(podcast_name: &str, episode: &Episode) -> Result<()> {
    let stdout = io::stdout();

    let mut path = get_podcast_dir()?;
    path.push(podcast_name);
    create_dir_if_not_exist(&path)?;

    if let Some(url) = episode.url() {
        if let Some(title) = episode.title() {
            let mut filename = title;
            filename.push_str(
                episode
                    .extension()
                    .chain_err(|| "unable to retrieve extension")?,
            );
            path.push(filename);
            if !path.exists() {
                {
                    let mut handle = stdout.lock();
                    writeln!(&mut handle, "Downloading: {:?}", &path).ok();
                }
                let mut file = File::create(&path).chain_err(|| UNABLE_TO_CREATE_FILE)?;
                let mut resp = reqwest::get(url).chain_err(|| UNABLE_TO_GET_HTTP_RESPONSE)?;
                let mut content: Vec<u8> = Vec::new();
                resp.read_to_end(&mut content)
                    .chain_err(|| UNABLE_TO_READ_RESPONSE_TO_END)?;
                file.write_all(&content)
                    .chain_err(|| UNABLE_TO_WRITE_FILE)?;
            } else {
                let mut handle = stdout.lock();
                writeln!(&mut handle, "File already exists: {:?}", &path).ok();
            }
        }
    }
    Ok(())
}

pub fn download_episode_by_name(
    state: &State,
    p_search: &str,
    e_search: &str,
    download_all: bool,
) -> Result<()> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)
                .chain_err(|| UNABLE_TO_RETRIEVE_PODCAST_BY_TITLE)?;
            let episodes = podcast.episodes();
            if download_all {
                episodes
                    .iter()
                    .filter(|ep| ep.title().is_some())
                    .filter(|ep| ep.title().unwrap().contains(e_search))
                    .for_each(|ep| {
                        download(podcast.title(), ep).unwrap_or_else(|_| {
                            eprintln!("Error downloading episode: {}", podcast.title())
                        });
                    })
            } else {
                let filtered_episodes: Vec<&Episode> = episodes
                    .iter()
                    .filter(|ep| ep.title().is_some())
                    .filter(|ep| {
                        ep.title()
                            .unwrap()
                            .to_lowercase()
                            .contains(&e_search.to_lowercase())
                    })
                    .collect();

                if let Some(ep) = filtered_episodes.first() {
                    download(podcast.title(), ep).chain_err(|| "unable to download episode")?;
                }
            }
        }
    }
    Ok(())
}

pub fn download_all(state: &State, p_search: &str) -> Result<()> {
    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;

    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let podcast = Podcast::from_title(&subscription.title)
                .chain_err(|| UNABLE_TO_RETRIEVE_PODCAST_BY_TITLE)?;
            podcast
                .download()
                .chain_err(|| "unable to download podcast")?;
        }
    }
    Ok(())
}

pub fn play_latest(state: &State, p_search: &str) -> Result<()> {
    let re_pod: Regex =
        Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;
    let mut path: PathBuf = get_xml_dir()?;
    DirBuilder::new()
        .recursive(true)
        .create(&path)
        .chain_err(|| UNABLE_TO_CREATE_DIRECTORY)?;
    for subscription in &state.subscriptions {
        if re_pod.is_match(&subscription.title) {
            let mut filename: String = subscription.title.clone();
            filename.push_str(".xml");
            path.push(filename);

            let mut file: File = File::open(&path).chain_err(|| UNABLE_TO_OPEN_FILE)?;
            let mut content: Vec<u8> = Vec::new();
            file.read_to_end(&mut content)
                .chain_err(|| "unable to read file to end")?;

            let podcast: Podcast = Podcast::from(
                Channel::read_from(content.as_slice())
                    .chain_err(|| UNABLE_TO_CREATE_CHANNEL_FROM_FILE)?,
            );
            let episodes = podcast.episodes();
            let episode = episodes[0].clone();

            filename = episode
                .title()
                .chain_err(|| "unable to retrieve episode name")?;
            filename.push_str(
                episode
                    .extension()
                    .chain_err(|| "unable to retrieve episode extension")?,
            );
            path = get_podcast_dir()?;
            path.push(podcast.title());
            path.push(filename);
            if path.exists() {
                launch_player(
                    path.to_str()
                        .chain_err(|| "unable to convert path to &str")?,
                )?;
            } else {
                launch_player(
                    episode
                        .url()
                        .chain_err(|| "unable to retrieve episode url")?,
                )?;
            }
            return Ok(());
        }
    }
    Ok(())
}

pub fn play_episode_by_num(state: &State, p_search: &str, ep_num_string: &str) -> Result<()> {
    let re_pod: Regex =
        Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;
    if let Ok(ep_num) = ep_num_string.parse::<usize>() {
        let mut path: PathBuf = get_xml_dir()?;
        if let Err(err) = DirBuilder::new().recursive(true).create(&path) {
            eprintln!(
                "Couldn't create directory: {}\nReason: {}",
                path.to_str().unwrap(),
                err
            );
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
                let episode = episodes[episodes.len() - ep_num].clone();

                filename = episode.title().unwrap();
                filename.push_str(episode.extension().unwrap());
                path = get_podcast_dir()?;
                path.push(podcast.title());
                path.push(filename);
                if path.exists() {
                    launch_player(path.to_str().chain_err(|| UNABLE_TO_CONVERT_TO_STR)?)?;
                } else {
                    launch_player(
                        episode
                            .url()
                            .chain_err(|| "unable to retrieve episode url")?,
                    )?;
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
        play_episode_by_name(state, p_search, ep_num_string)
            .chain_err(|| "Failed to play episode by name.")?;
    }
    Ok(())
}

pub fn play_episode_by_name(state: &State, p_search: &str, ep_string: &str) -> Result<()> {
    let re_pod: Regex =
        Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;
    let mut path: PathBuf = get_xml_dir()?;
    if let Err(err) = DirBuilder::new().recursive(true).create(&path) {
        eprintln!(
            "Couldn't create directory: {}\nReason: {}",
            path.to_str().unwrap(),
            err
        );
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
                filename.push_str(episode.extension().unwrap());
                path = get_podcast_dir()?;
                path.push(podcast.title());
                path.push(filename);
                if path.exists() {
                    launch_player(path.to_str().chain_err(|| UNABLE_TO_CONVERT_TO_STR)?)?;
                } else {
                    launch_player(
                        episode
                            .url()
                            .chain_err(|| "unable to retrieve episode url")?,
                    )?;
                }
            }
            return Ok(());
        }
    }
    Ok(())
}

pub fn check_for_update(version: &str) -> Result<()> {
    println!("Checking for updates...");
    let resp: String =
        reqwest::get("https://raw.githubusercontent.com/njaremko/podcast/master/Cargo.toml")
            .chain_err(|| UNABLE_TO_GET_HTTP_RESPONSE)?
            .text()
            .chain_err(|| "unable to convert response to text")?;

    let config = resp
        .parse::<toml::Value>()
        .chain_err(|| "unable to parse toml")?;
    let latest = config["package"]["version"]
        .as_str()
        .chain_err(|| UNABLE_TO_CONVERT_TO_STR)?;
    if version != latest {
        println!("New version avaliable: {} -> {}", version, latest);
    }
    Ok(())
}

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
        match err.kind() {
            io::ErrorKind::NotFound => {
                eprintln!("Couldn't open mpv\nTrying vlc...");
            }
            _ => eprintln!("Error: {}", err),
        }
    }
    Ok(())
}

fn launch_vlc(url: &str) -> Result<()> {
    if let Err(err) = Command::new("vlc").args(&["-I ncurses", url]).status() {
        match err.kind() {
            io::ErrorKind::NotFound => {
                eprintln!("Couldn't open vlc...aborting");
            }
            _ => eprintln!("Error: {}", err),
        }
    }
    Ok(())
}

pub fn remove_podcast(state: &mut State, p_search: &str) -> Result<()> {
    if p_search == "*" {
        state.subscriptions = vec![];
        return delete_all();
    }

    let re_pod = Regex::new(&format!("(?i){}", &p_search)).chain_err(|| UNABLE_TO_PARSE_REGEX)?;

    for subscription in 0..state.subscriptions.len() {
        let title = state.subscriptions[subscription].title.clone();
        if re_pod.is_match(&title) {
            state.subscriptions.remove(subscription);
            delete(&title)?;
        }
    }
    Ok(())
}

pub fn print_completion(app: &mut App, arg: &str) {
    match arg {
        "zsh" => app.gen_completions_to("podcast", Shell::Zsh, &mut io::stdout()),
        "bash" => app.gen_completions_to("podcast", Shell::Bash, &mut io::stdout()),
        "powershell" => app.gen_completions_to("podcast", Shell::PowerShell, &mut io::stdout()),
        "fish" => app.gen_completions_to("podcast", Shell::Fish, &mut io::stdout()),
        "elvish" => app.gen_completions_to("podcast", Shell::Elvish, &mut io::stdout()),
        other => eprintln!("Completions are not available for {}", other),
    }
}
