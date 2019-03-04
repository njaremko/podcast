use crate::download;
use crate::errors::*;
use crate::structs::*;
use crate::utils::*;

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};

use clap::App;
use clap::Shell;
use rayon::prelude::*;
use regex::Regex;
use reqwest;
use rss::Channel;
use std::path::PathBuf;
use toml;

pub fn list_episodes(search: &str) -> Result<()> {
    let re = Regex::new(&format!("(?i){}", &search))?;
    let path = get_xml_dir()?;
    create_dir_if_not_exist(&path)?;

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

pub fn update_subscription(sub: &mut Subscription) -> Result<()> {
    println!("Updating {}", sub.title);
    let mut path: PathBuf = get_podcast_dir()?;
    path.push(&sub.title);
    create_dir_if_not_exist(&path)?;

    let mut titles = HashSet::new();
    for entry in fs::read_dir(&path)? {
        let unwrapped_entry = &entry?;
        titles.insert(trim_extension(
            &unwrapped_entry.file_name().into_string().unwrap(),
        ));
    }

    let resp = reqwest::get(&sub.url)?;
    let podcast = Podcast::from(Channel::read_from(BufReader::new(resp))?);

    let mut podcast_rss_path = get_xml_dir()?;
    podcast_rss_path.push(podcast.title());
    podcast_rss_path.set_extension("xml");

    let file = File::create(&podcast_rss_path)?;
    (*podcast).write_to(BufWriter::new(file))?;

    if sub.num_episodes < podcast.episodes().len() {
        podcast.episodes()[..podcast.episodes().len() - sub.num_episodes]
            .par_iter()
            .map(|ep| download::download(podcast.title(), ep))
            .flat_map(|e| e.err())
            .for_each(|err| eprintln!("Error: {}", err));
    }
    sub.num_episodes = podcast.episodes().len();
    Ok(())
}

pub fn update_rss(state: &mut State) {
    println!("Checking for new episodes...");
    let _result: Vec<Result<()>> = state
        .subscriptions_mut()
        .par_iter_mut()
        .map(|sub: &mut Subscription| update_subscription(sub))
        .collect();
    println!("Done.");
}

pub fn list_subscriptions(state: &State) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for subscription in state.subscriptions() {
        writeln!(&mut handle, "{}", subscription.title())?;
    }
    Ok(())
}

pub fn check_for_update(version: &str) -> Result<()> {
    println!("Checking for updates...");
    let resp: String =
        reqwest::get("https://raw.githubusercontent.com/njaremko/podcast/master/Cargo.toml")?
            .text()?;

    let config = resp.parse::<toml::Value>()?;
    let latest = config["package"]["version"]
        .as_str()
        .expect(&format!("Cargo.toml didn't have a version {:?}", config));
    if version != latest {
        println!("New version available: {} -> {}", version, latest);
    }
    Ok(())
}

pub fn remove_podcast(state: &mut State, p_search: &str) -> Result<()> {
    if p_search == "*" {
        state.subscriptions = vec![];
        return delete_all();
    }

    let re_pod = Regex::new(&format!("(?i){}", &p_search))?;

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
