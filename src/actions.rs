use crate::download;
use crate::errors::*;
use crate::structs::*;
use crate::utils;

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};

use clap::App;
use clap::Shell;
use regex::Regex;
use reqwest;
use rss::Channel;
use semver_parser::version;
use std::path::PathBuf;
use toml;

pub fn list_episodes(search: &str) -> Result<()> {
    let re = Regex::new(&format!("(?i){}", &search))?;
    let path = utils::get_xml_dir()?;
    utils::create_dir_if_not_exist(&path)?;

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        if re.is_match(&entry.file_name().into_string().unwrap()) {
            let file = File::open(&entry.path())?;
            let channel = Channel::read_from(BufReader::new(file))?;
            let podcast = Podcast::from(channel);
            let episodes = podcast.episodes();
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            episodes
                .iter()
                .filter(|ep| ep.title().is_some())
                .enumerate()
                .for_each(|(num, ep)| {
                    writeln!(
                        &mut handle,
                        "({}) {}",
                        episodes.len() - num,
                        ep.title().unwrap()
                    )
                    .ok();
                });
            return Ok(());
        }
    }
    Ok(())
}

pub async fn update_subscription(sub: &mut Subscription, config: Option<Config>) -> Result<()> {
    println!("Updating {}", sub.title);
    let mut path: PathBuf = utils::get_podcast_dir()?;
    path.push(&sub.title);
    utils::create_dir_if_not_exist(&path)?;

    let mut titles = HashSet::new();
    for entry in fs::read_dir(&path)? {
        let unwrapped_entry = &entry?;
        titles.insert(utils::trim_extension(
            &unwrapped_entry.file_name().into_string().unwrap(),
        ));
    }

    let resp = reqwest::get(&sub.url).await?.bytes().await?;
    let podcast = Podcast::from(Channel::read_from(BufReader::new(&resp[..]))?);

    let mut podcast_rss_path = utils::get_xml_dir()?;
    let title = utils::append_extension(podcast.title(), "xml");
    podcast_rss_path.push(title);

    let file = File::create(&podcast_rss_path)?;
    (*podcast).write_to(BufWriter::new(file))?;

    if sub.num_episodes < podcast.episodes().len() {
        let mut d_vec = vec![];
        let subscription_limit = config.unwrap_or_else(|| Config::default()).download_subscription_limit.unwrap_or(-1);
        let mut episodes = podcast.episodes()[..podcast.episodes().len() - sub.num_episodes].to_vec();
        episodes.reverse();
        if 0 < subscription_limit {
            for ep in episodes.iter().take(subscription_limit as usize) {
                d_vec.push(download::download(podcast.title().into(), ep.clone()));
            }
        } else {
            for ep in episodes.iter() {
                d_vec.push(download::download(podcast.title().into(), ep.clone()));
            }
        }
        for c in futures::future::join_all(d_vec).await.iter() {
            if let Err(err) = c {
                println!("Error: {}", err);
            }
        }
    }
    sub.num_episodes = podcast.episodes().len();
    Ok(())
}

pub async fn update_rss(state: &mut State, config: Option<Config>) -> Result<()> {
    println!("Checking for new episodes...");
    let mut d_vec = vec![];
    for sub in state.subscriptions_mut() {
        d_vec.push(update_subscription(sub, config));
    }
    for c in futures::future::join_all(d_vec).await.iter() {
        if let Err(err) = c {
            println!("Error: {}", err);
        }
    }
    println!("Done.");
    Ok(())
}

pub fn list_subscriptions(state: &State) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for subscription in state.subscriptions() {
        writeln!(&mut handle, "{}", subscription.title())?;
    }
    Ok(())
}

pub async fn check_for_update(version: &str) -> Result<()> {
    println!("Checking for updates...");
    let resp: String =
        reqwest::get("https://raw.githubusercontent.com/njaremko/podcast/master/Cargo.toml")
            .await?
            .text()
            .await?;

    let config = resp.parse::<toml::Value>()?;
    let latest = config["package"]["version"]
        .as_str()
        .unwrap_or_else(|| panic!("Cargo.toml didn't have a version {:?}", config));
    let local_version = match version::parse(&version) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse version {}: {}", &version, e);
            return Ok(());
        }
    };
    let remote_version = match version::parse(&latest) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse version {}: {}", &version, e);
            return Ok(());
        }
    };
    if local_version < remote_version {
        println!("New version available: {} -> {}", version, latest);
    }
    Ok(())
}

pub fn remove_podcast(state: &mut State, p_search: &str) -> Result<()> {
    if p_search == "*" {
        state.subscriptions = vec![];
        return utils::delete_all();
    }

    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

    for subscription in 0..state.subscriptions.len() {
        let title = state.subscriptions[subscription].title.clone();
        if re_pod.is_match(&title) {
            state.subscriptions.remove(subscription);
            utils::delete(&title)?;
        }
    }
    Ok(())
}

pub fn print_completion(app: &mut App, arg: &str) {
    let command_name = "podcast";
    match arg {
        "zsh" => {
            app.gen_completions_to(command_name, Shell::Zsh, &mut io::stdout());
        }
        "bash" => {
            app.gen_completions_to(command_name, Shell::Bash, &mut io::stdout());
        }
        "powershell" => {
            app.gen_completions_to(command_name, Shell::PowerShell, &mut io::stdout());
        }
        "fish" => {
            app.gen_completions_to(command_name, Shell::Fish, &mut io::stdout());
        }
        "elvish" => {
            app.gen_completions_to(command_name, Shell::Elvish, &mut io::stdout());
        }
        other => {
            println!("Completions are not available for {}", other);
        }
    }
}
