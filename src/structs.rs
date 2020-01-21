use super::actions::*;
use super::utils::*;
use core::ops::Deref;
use crate::errors::*;

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};

use chrono::prelude::*;
use regex::Regex;
use rss::{Channel, Item};
use serde_json;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
const ESCAPE_REGEX: &str = r"/";
#[cfg(target_os = "linux")]
const ESCAPE_REGEX: &str = r"/";
#[cfg(target_os = "windows")]
const ESCAPE_REGEX: &str = r#"[\\/:*?"<>|]"#;

lazy_static! {
    static ref FILENAME_ESCAPE: Regex = Regex::new(ESCAPE_REGEX).unwrap();
}

fn create_new_config_file(path: &PathBuf) -> Result<Config> {
    writeln!(
        io::stdout().lock(),
        "Creating new config file at {:?}",
        &path
    )
    .ok();
    let file = File::create(&path)?;
    let config = Config {
        auto_download_limit: Some(1),
        download_subscription_limit: Some(1),
    };
    serde_yaml::to_writer(file, &config)?;
    Ok(config)
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub auto_download_limit: Option<i64>,
    pub download_subscription_limit: Option<i64>,
}

impl Config {
    pub fn new() -> Result<Config> {
        let mut path = get_podcast_dir()?;
        path.push(".config.yaml");
        let config = if path.exists() {
            let file = File::open(&path)?;
            match serde_yaml::from_reader(file) {
                Ok(config) => config,
                Err(err) => {
                    let mut new_path = path.clone();
                    new_path.set_extension("yaml.bk");
                    let stderr = io::stderr();
                    let mut handle = stderr.lock();
                    writeln!(
                        &mut handle,
                        "{}\nFailed to open config file, moving to {:?}",
                        err, &new_path
                    )
                    .ok();
                    fs::rename(&path, new_path)?;
                    create_new_config_file(&path)?
                }
            }
        } else {
            create_new_config_file(&path)?
        };
        Ok(config)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    pub title: String,
    pub url: String,
    pub num_episodes: usize,
}

impl Subscription {
    pub fn title(&self) -> &str {
        &self.title
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub version: String,
    pub last_run_time: DateTime<Utc>,
    pub subscriptions: Vec<Subscription>,
}

impl State {
    pub async fn new(version: &str) -> Result<State> {
        let path = get_sub_file()?;
        if path.exists() {
            let file = File::open(&path)?;
            let mut state: State = serde_json::from_reader(BufReader::new(&file))?;
            state.version = String::from(version);
            // Check if a day has passed (86400 seconds) since last launch
            if 86400
                < Utc::now()
                    .signed_duration_since(state.last_run_time)
                    .num_seconds()
            {
                let config = Config::new()?;
                update_rss(&mut state, Some(config)).await?;
                check_for_update(&state.version).await?;
            }
            state.last_run_time = Utc::now();
            state.save()?;
            Ok(state)
        } else {
            writeln!(io::stdout().lock(), "Creating new file: {:?}", &path).ok();
            Ok(State {
                version: String::from(version),
                last_run_time: Utc::now(),
                subscriptions: Vec::new(),
            })
        }
    }

    pub fn subscriptions(&self) -> &[Subscription] {
        &self.subscriptions
    }

    pub fn subscriptions_mut(&mut self) -> &mut [Subscription] {
        &mut self.subscriptions
    }

    pub async fn subscribe(&mut self, url: &str) -> Result<()> {
        let mut set = HashSet::new();
        for sub in self.subscriptions() {
            set.insert(sub.title.clone());
        }
        let resp = reqwest::get(url).await?.bytes().await?;
        let channel = Channel::read_from(BufReader::new(&resp[..]))?;
        let podcast = Podcast::from(channel);
        if !set.contains(podcast.title()) {
            self.subscriptions.push(Subscription {
                title: String::from(podcast.title()),
                url: String::from(url),
                num_episodes: podcast.episodes().len(),
            });
        }
        self.save()
    }

    pub fn save(&self) -> Result<()> {
        let mut path = get_sub_file()?;
        path.set_extension("json.tmp");
        let file = File::create(&path)?;
        serde_json::to_writer(BufWriter::new(file), self)?;
        fs::rename(&path, get_sub_file()?)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Podcast(Channel);

impl From<Channel> for Podcast {
    fn from(channel: Channel) -> Podcast {
        Podcast(channel)
    }
}

impl Deref for Podcast {
    type Target = Channel;

    fn deref(&self) -> &Channel {
        &self.0
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
        Ok(Podcast::from(Channel::from_url(url)?))
    }

    pub fn from_title(title: &str) -> Result<Podcast> {
        let mut path = get_xml_dir()?;
        let mut filename = String::from(title);
        filename.push_str(".xml");
        path.push(filename);

        let file = File::open(&path)?;
        Ok(Podcast::from(Channel::read_from(BufReader::new(file))?))
    }

    pub fn episodes(&self) -> Vec<Episode> {
        let mut result = Vec::new();
        for item in self.0.items().to_owned() {
            result.push(Episode::from(item));
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Episode(Item);

impl From<Item> for Episode {
    fn from(item: Item) -> Episode {
        Episode(item)
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

    pub fn extension(&self) -> Option<String> {
        match self.0.enclosure()?.mime_type() {
            "audio/mpeg" => Some("mp3".into()),
            "audio/mp4" => Some("m4a".into()),
            "audio/aac" => Some("m4a".into()),
            "audio/ogg" => Some("ogg".into()),
            "audio/vorbis" => Some("ogg".into()),
            "audio/opus" => Some("opus".into()),
            _ => find_extension(self.url().unwrap()),
        }
    }
}
