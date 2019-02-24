use super::actions::*;
use super::utils::*;
use crate::errors::*;

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};

use chrono::prelude::*;
use rayon::prelude::*;
use regex::Regex;
use rss::{Channel, Item};
use serde_json;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
static ESCAPE_REGEX: &str = r"/";
#[cfg(target_os = "linux")]
static ESCAPE_REGEX: &str = r"/";
#[cfg(target_os = "windows")]
static ESCAPE_REGEX: &str = r#"[\\/:*?"<>|]"#;

lazy_static! {
    static ref FILENAME_ESCAPE: Regex = Regex::new(ESCAPE_REGEX).unwrap();
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub auto_download_limit: i64,
}

impl Config {
    pub fn new() -> Result<Config> {
        let mut path = get_podcast_dir()?;
        path.push(".config.yaml");
        let config = if path.exists() {
            let file = File::open(&path).chain_err(|| UNABLE_TO_OPEN_FILE)?;
            match serde_yaml::from_reader(file) {
                Ok(config) => config,
                Err(err) => {
                    let mut new_path = path.clone();
                    new_path.set_extension("yaml.bk");
                    eprintln!("{}", err);
                    eprintln!("Failed to open config file, moving to {:?}", &new_path);
                    fs::rename(&path, new_path)
                        .chain_err(|| "Failed to move old config file...")?;
                    create_new_config_file(&path)?
                }
            }
        } else {
            create_new_config_file(&path)?
        };
        Ok(config)
    }
}

fn create_new_config_file(path: &PathBuf) -> Result<Config> {
    println!("Creating new config file at {:?}", &path);
    let download_limit = 1;
    let file = File::create(&path).chain_err(|| UNABLE_TO_CREATE_FILE)?;
    let config = Config {
        auto_download_limit: download_limit,
    };
    serde_yaml::to_writer(file, &config).chain_err(|| UNABLE_TO_WRITE_FILE)?;
    Ok(config)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    pub title: String,
    pub url: String,
    pub num_episodes: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub last_run_time: DateTime<Utc>,
    pub subscriptions: Vec<Subscription>,
}

impl State {
    pub fn new(version: &str) -> Result<State> {
        let path = get_sub_file()?;
        if path.exists() {
            let file = File::open(&path).chain_err(|| UNABLE_TO_OPEN_FILE)?;
            let mut state: State = match serde_json::from_reader(&file) {
                Ok(val) => val,
                // This will happen if the struct has changed between versions
                Err(_) => {
                    let v: serde_json::Value = serde_json::from_reader(&file)
                        .chain_err(|| "unable to read json from string")?;
                    State {
                        version: String::from(version),
                        last_run_time: Utc::now(),
                        subscriptions: match serde_json::from_value(v["subscriptions"].clone()) {
                            Ok(val) => val,
                            Err(_) => serde_json::from_value(v["subs"].clone())
                                .chain_err(|| "unable to parse value from json")?,
                        },
                    }
                }
            };
            state.version = String::from(version);
            // Check if a day has passed (86400 seconds) since last launch
            if 86400 < Utc::now()
                .signed_duration_since(state.last_run_time)
                .num_seconds()
            {
                update_rss(&mut state);
                check_for_update(&state.version)?;
            }
            state.last_run_time = Utc::now();
            state.save()?;
            Ok(state)
        } else {
            println!("Creating new file {:?}", &path);
            Ok(State {
                version: String::from(version),
                last_run_time: Utc::now(),
                subscriptions: Vec::new(),
            })
        }
    }

    pub fn subscribe(&mut self, url: &str) -> Result<()> {
        let mut set = BTreeSet::new();
        for sub in self.subscriptions() {
            set.insert(sub.title.clone());
        }
        let podcast = Podcast::from(Channel::from_url(url).unwrap());
        if !set.contains(podcast.title()) {
            self.subscriptions.push(Subscription {
                title: String::from(podcast.title()),
                url: String::from(url),
                num_episodes: podcast.episodes().len(),
            });
        }
        self.save()
    }

    pub fn subscriptions(&self) -> &[Subscription] {
        &self.subscriptions
    }

    pub fn subscriptions_mut(&mut self) -> &mut [Subscription] {
        &mut self.subscriptions
    }

    pub fn save(&self) -> Result<()> {
        let mut path = get_sub_file()?;
        path.set_extension("json.tmp");
        let serialized = serde_json::to_string(self).chain_err(|| "unable to serialize state")?;
        {
            let mut file = File::create(&path).chain_err(|| UNABLE_TO_CREATE_FILE)?;
            file.write_all(serialized.as_bytes())
                .chain_err(|| UNABLE_TO_WRITE_FILE)?;
        }
        let sub_file_path = get_sub_file()?;
        fs::rename(&path, &sub_file_path)
            .chain_err(|| format!("unable to rename file {:?} to {:?}", &path, &sub_file_path))?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Podcast(Channel);

#[derive(Clone, Debug)]
pub struct Episode(Item);

impl From<Channel> for Podcast {
    fn from(channel: Channel) -> Podcast {
        Podcast(channel)
    }
}

impl From<Item> for Episode {
    fn from(item: Item) -> Episode {
        Episode(item)
    }
}

impl Podcast {
    pub fn title(&self) -> &str {
        self.0.title()
    }

    #[allow(dead_code)]
    pub fn url(&self) -> &str {
        self.0.link()
    }

    #[allow(dead_code)]
    pub fn from_url(url: &str) -> Result<Podcast> {
        Ok(Podcast::from(
            Channel::from_url(url).chain_err(|| UNABLE_TO_CREATE_CHANNEL_FROM_RESPONSE)?,
        ))
    }

    pub fn from_title(title: &str) -> Result<Podcast> {
        let mut path = get_xml_dir()?;
        let mut filename = String::from(title);
        filename.push_str(".xml");
        path.push(filename);

        let file = File::open(&path).chain_err(|| UNABLE_TO_OPEN_FILE)?;
        Ok(Podcast::from(
            Channel::read_from(BufReader::new(file))
                .chain_err(|| UNABLE_TO_CREATE_CHANNEL_FROM_FILE)?,
        ))
    }

    pub fn episodes(&self) -> Vec<Episode> {
        let mut result = Vec::new();
        for item in self.0.items().to_owned() {
            result.push(Episode::from(item));
        }
        result
    }

    pub fn download(&self) -> Result<()> {
        print!(
            "You are about to download all episodes of {} (y/n): ",
            self.title()
        );
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .chain_err(|| "unable to read stdin")?;
        if input.to_lowercase().trim() != "y" {
            return Ok(());
        }

        let mut path = get_podcast_dir()?;
        path.push(self.title());

        match already_downloaded(self.title()) {
            Ok(downloaded) => {
                self.episodes().par_iter().for_each(|i| {
                    if let Some(ep_title) = i.title() {
                        if !downloaded.contains(&ep_title) {
                            if let Err(err) = download(self.title(), i) {
                                eprintln!("{}", err);
                            }
                        }
                    }
                });
            }
            Err(_) => {
                self.episodes().par_iter().for_each(|i| {
                    if let Err(err) = download(self.title(), i) {
                        eprintln!("{}", err);
                    }
                });
            }
        }

        Ok(())
    }

    pub fn download_specific(&self, episode_numbers: &[usize]) -> Result<()> {
        let mut path = get_podcast_dir()?;
        path.push(self.title());

        let downloaded = already_downloaded(self.title())?;
        let episodes = self.episodes();

        episode_numbers.par_iter().for_each(|ep_num| {
            if let Some(ep_title) = episodes[episodes.len() - ep_num].title() {
                if !downloaded.contains(&ep_title) {
                    if let Err(err) = download(self.title(), &episodes[episodes.len() - ep_num]) {
                        eprintln!("{}", err);
                    }
                }
            }
        });
        Ok(())
    }
}

impl Episode {
    pub fn title(&self) -> Option<String> {
        Some(
            FILENAME_ESCAPE
                .replace_all(self.0.title()?, "_")
                .to_string(),
        )
    }

    pub fn url(&self) -> Option<&str> {
        match self.0.enclosure() {
            Some(val) => Some(val.url()),
            None => None,
        }
    }

    pub fn extension(&self) -> Option<&str> {
        match self.0.enclosure()?.mime_type() {
            "audio/mpeg" => Some(".mp3"),
            "audio/mp4" => Some(".m4a"),
            "audio/ogg" => Some(".ogg"),
            _ => find_extension(self.url().unwrap()),
        }
    }
}
