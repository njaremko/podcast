use super::actions::*;
use super::utils::*;
use anyhow::Result;
use core::ops::Deref;

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};

use crate::{download, utils};
use bloom::ASMS;
use chrono::prelude::*;
use regex::Regex;
use reqwest::header;
use rss::{Channel, Item};
use semver_parser::version;
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

/// This information is persisted to disk as part of PublicState
/// and allows for configuration of the CLI
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub auto_download_limit: Option<i64>,
    pub download_subscription_limit: Option<i64>,
    pub quiet: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_download_limit: Some(1),
            download_subscription_limit: Some(1),
            quiet: Some(false),
        }
    }
}

impl Config {
    pub fn load() -> Result<Option<Config>> {
        let mut path = get_podcast_dir()?;
        path.push(".config.yaml");
        if path.exists() {
            let file = File::open(&path)?;
            return Ok(Some(serde_yaml::from_reader(file)?));
        }
        Ok(None)
    }
}

/// This is persisted to disk and represents each subscription and it's last known state
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

/// This struct is what is serialized to disk
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicState {
    pub version: String,
    pub last_run_time: DateTime<Utc>,
    pub config: Config,
    pub subscriptions: Vec<Subscription>,
}

impl From<State> for PublicState {
    fn from(internal_state: State) -> Self {
        PublicState {
            version: internal_state.version,
            last_run_time: internal_state.last_run_time,
            config: internal_state.config,
            subscriptions: internal_state.subscriptions,
        }
    }
}

impl PublicState {
    pub fn save(&self) -> Result<()> {
        let mut path = config_path()?;
        path.set_extension("json.tmp");
        let file = File::create(&path)?;
        serde_json::to_writer_pretty(BufWriter::new(file), self)?;
        fs::rename(&path, config_path()?)?;
        Ok(())
    }
}

/// Internal state across the application, cannot be serialized
#[derive(Clone, Debug)]
pub struct State {
    pub version: String,
    pub last_run_time: DateTime<Utc>,
    pub config: Config,
    pub subscriptions: Vec<Subscription>,
    pub client: reqwest::Client,
}

/// Struct used to parse state from disk, handles missing
/// configuration and populates with sensible defaults
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ParseState {
    version: Option<String>,
    last_run_time: Option<DateTime<Utc>>,
    config: Option<Config>,
    subscriptions: Option<Vec<Subscription>>,
}

impl From<ParseState> for State {
    fn from(internal_state: ParseState) -> Self {
        State {
            version: internal_state.version.unwrap(),
            last_run_time: internal_state.last_run_time.unwrap_or_else(|| Utc::now()),
            config: internal_state.config.unwrap_or_else(|| Config::default()),
            subscriptions: internal_state.subscriptions.unwrap_or_else(|| vec![]),
            client: reqwest::Client::new(),
        }
    }
}

impl State {
    pub async fn new(version: &str, config: Config) -> Result<State> {
        let config_path = config_path()?;
        let legacy_subscription_path = get_sub_file()?;

        // TODO This moves legacy file into new location. Remove this.
        if legacy_subscription_path.exists() {
            std::fs::rename(&legacy_subscription_path, &config_path)?;
        }

        if config_path.exists() {
            let file = File::open(&config_path)?;
            // Read the file into an internal struct that allows optionally missing fields
            let parse_state: ParseState = serde_json::from_reader(BufReader::new(&file))?;

            // Convert to our public state that has sensible default for non-present fields
            let mut state: State = parse_state.into();

            // Override version to the version currently running
            state.version = String::from(version);

            // Check if a day has passed since last launch
            if 0 < Utc::now()
                .signed_duration_since(state.last_run_time)
                .num_days()
            {
                state.check_for_update().await?;
                state.update_rss().await?;
            }

            // Update last run time and persist config
            state.last_run_time = Utc::now();

            Ok(state)
        } else {
            writeln!(io::stdout().lock(), "Creating new file: {:?}", &config_path).ok();
            Ok(State {
                version: String::from(version),
                last_run_time: Utc::now(),
                subscriptions: Vec::new(),
                config,
                client: reqwest::Client::new(),
            })
        }
    }

    pub async fn subscribe(&mut self, url: &str) -> Result<()> {
        // Make a bloom filter and populate it with subscription titles
        let existing_subscriptions = if self.subscriptions.is_empty() {
            10
        } else {
            self.subscriptions.len()
        };

        let mut bloom_filter = bloom::BloomFilter::with_rate(0.1, existing_subscriptions as u32);
        for sub in &self.subscriptions {
            bloom_filter.insert(&sub.title);
        }

        // Fetch provided podcast RSS feed
        let resp = reqwest::get(url).await?.bytes().await?;

        // Parse the response into a podcast struct
        let channel = Channel::read_from(BufReader::new(&resp[..]))?;
        let podcast = Podcast::from(channel);

        // Check if the podcast already exists in our subscriptions
        if !bloom_filter.contains(&podcast.title()) {
            self.subscriptions.push(Subscription {
                title: String::from(podcast.title()),
                url: String::from(url),
                num_episodes: podcast.episodes().len(),
            });
        }
        let episodes = download::download_rss(self, url).await?;
        download::download_episodes(episodes).await?;
        Ok(())
    }

    pub async fn update_rss(&mut self) -> Result<()> {
        println!("Checking for new episodes...");
        let mut d_vec = vec![];
        for (index, sub) in self.subscriptions.iter().enumerate() {
            d_vec.push(update_subscription(&self, index, sub, &self.config));
        }
        let new_subscriptions = futures::future::join_all(d_vec).await;
        for c in &new_subscriptions {
            match c {
                Ok([index, new_ep_count]) => {
                    self.subscriptions[*index].num_episodes = *new_ep_count;
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
        println!("Done.");
        Ok(())
    }

    pub async fn check_for_update(&self) -> Result<()> {
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
        let local_version = match version::parse(&self.version) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse version {}: {}", &self.version, e);
                return Ok(());
            }
        };
        let remote_version = match version::parse(&latest) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse version {}: {}", &self.version, e);
                return Ok(());
            }
        };
        if local_version < remote_version {
            println!("New version available: {} -> {}", &self.version, latest);
        }
        Ok(())
    }
}

/// Represent an intention to download a file
#[derive(Clone, Debug, PartialEq)]
pub struct Download {
    pub title: String,
    pub path: PathBuf,
    pub url: String,
    pub size: u64,
}

impl Download {
    pub async fn new(
        state: &State,
        podcast: &Podcast,
        episode: &Episode,
    ) -> Result<Option<Download>> {
        let mut path = utils::get_podcast_dir()?;
        path.push(podcast.title());
        utils::create_dir_if_not_exist(&path)?;
        if let (Some(mut title), Some(url)) = (episode.title(), episode.url()) {
            if let Some(ext) = episode.extension() {
                title = utils::append_extension(&title, &ext);
            }
            path.push(&title);

            let head_resp = state.client.head(url).send().await?;
            let total_size = head_resp
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse().ok())
                .unwrap_or(0);

            if !path.exists() {
                return Ok(Some(Download {
                    title,
                    path,
                    url: url.into(),
                    size: total_size,
                }));
            }
        }
        Ok(None)
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
    pub async fn from_url(url: &str) -> Result<Podcast> {
        let content = reqwest::get(url).await?.bytes().await?;
        Ok(Podcast::from(Channel::read_from(&content[..])?))
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
