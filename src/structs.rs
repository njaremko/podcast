use super::actions::*;
use super::utils::*;
use crate::errors::*;

use std::collections::BTreeSet;
use std::fs::{self, DirBuilder, File};
use std::io::{self, BufReader, Read, Write};

use chrono::prelude::*;
use rayon::prelude::*;
use regex::Regex;
use reqwest;
use rss::{Channel, Item};
use serde_json;
use yaml_rust::YamlLoader;

#[cfg(target_os = "macos")]
static ESCAPE_REGEX: &str = r"/";
#[cfg(target_os = "linux")]
static ESCAPE_REGEX: &str = r"/";
#[cfg(target_os = "windows")]
static ESCAPE_REGEX: &str = r#"[\\/:*?"<>|]"#;

lazy_static! {
    static ref FILENAME_ESCAPE: Regex = Regex::new(ESCAPE_REGEX).unwrap();
}

pub struct Config {
    pub auto_download_limit: i64,
}

impl Config {
    pub fn new() -> Result<Config> {
        let mut path = get_podcast_dir()?;
        let mut download_limit = 1;
        path.push(".config");
        if path.exists() {
            let mut s = String::new();
            File::open(&path)
                .chain_err(|| UNABLE_TO_OPEN_FILE)?
                .read_to_string(&mut s)
                .chain_err(|| UNABLE_TO_READ_FILE_TO_STRING)?;
            let config =
                YamlLoader::load_from_str(&s).chain_err(|| "unable to load yaml from string")?;
            if !config.is_empty() {
                let doc = &config[0];
                if let Some(val) = doc["auto_download_limit"].as_i64() {
                    download_limit = val;
                }
            }
        } else {
            let mut file = File::create(&path).chain_err(|| UNABLE_TO_CREATE_FILE)?;
            file.write_all(b"auto_download_limit: 1")
                .chain_err(|| UNABLE_TO_WRITE_FILE)?;
        }
        Ok(Config {
            auto_download_limit: download_limit,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Subscription {
    pub title: String,
    pub url: String,
    pub num_episodes: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub version: String,
    pub last_run_time: DateTime<Utc>,
    pub subscriptions: Vec<Subscription>,
}

impl State {
    pub fn new(version: &str) -> Result<State> {
        let mut path = get_podcast_dir()?;
        path.push(".subscriptions");
        if path.exists() {
            let mut s = String::new();
            {
                let mut file = File::open(&path).chain_err(|| UNABLE_TO_OPEN_FILE)?;
                file.read_to_string(&mut s)
                    .chain_err(|| UNABLE_TO_READ_FILE_TO_STRING)?;
            }
            let mut state: State = match serde_json::from_str(&s) {
                Ok(val) => val,
                // This will happen if the struct has changed between versions
                Err(_) => {
                    let v: serde_json::Value =
                        serde_json::from_str(&s).chain_err(|| "unable to read json from string")?;
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
            if Utc::now()
                .signed_duration_since(state.last_run_time)
                .num_seconds()
                > 86400
            {
                update_rss(&mut state);
                check_for_update(&state.version)?;
            }
            state.last_run_time = Utc::now();
            state.save()?;
            Ok(state)
        } else {
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
            set.insert(sub.title);
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

    pub fn subscriptions(&self) -> Vec<Subscription> {
        self.subscriptions.clone()
    }

    pub fn save(&self) -> Result<()> {
        let mut path = get_podcast_dir()?;
        path.push(".subscriptions.tmp");
        let serialized = serde_json::to_string(self).chain_err(|| "unable to serialize state")?;
        {
            let mut file = File::create(&path).chain_err(|| UNABLE_TO_CREATE_FILE)?;
            file.write_all(serialized.as_bytes())
                .chain_err(|| UNABLE_TO_WRITE_FILE)?;
        }
        fs::rename(&path, get_sub_file()?).chain_err(|| "unable to rename file")?;
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

    pub fn delete(title: &str) -> Result<()> {
        let mut path = get_xml_dir()?;
        let mut filename = String::from(title);
        filename.push_str(".xml");
        path.push(filename);

        fs::remove_file(path).chain_err(|| UNABLE_TO_REMOVE_FILE)
    }

    pub fn delete_all() -> Result<()> {
        let path = get_xml_dir()?;
        fs::remove_dir_all(path).chain_err(|| UNABLE_TO_READ_DIRECTORY)
    }

    pub fn episodes(&self) -> Vec<Episode> {
        let mut result = Vec::new();
        for item in self.0.items().to_vec() {
            result.push(Episode::from(item));
        }
        result
    }

    pub fn download(&self) -> Result<()> {
        print!("You are about to download all episodes (y/n): ");
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
                            if let Err(err) = i.download(self.title()) {
                                eprintln!("{}", err);
                            }
                        }
                    }
                });
            }
            Err(_) => {
                self.episodes().par_iter().for_each(|i| {
                    if let Err(err) = i.download(self.title()) {
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
                    if let Err(err) = episodes[episodes.len() - ep_num].download(self.title()) {
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

    pub fn download(&self, podcast_name: &str) -> Result<()> {
        let stdout = io::stdout();

        let mut path = get_podcast_dir()?;
        path.push(podcast_name);
        DirBuilder::new()
            .recursive(true)
            .create(&path)
            .chain_err(|| UNABLE_TO_CREATE_DIRECTORY)?;

        if let Some(url) = self.url() {
            if let Some(title) = self.title() {
                let mut filename = title;
                filename.push_str(
                    self.extension()
                        .chain_err(|| "unable to retrieve extension")?,
                );
                path.push(filename);
                if !path.exists() {
                    {
                        let mut handle = stdout.lock();
                        writeln!(&mut handle, "Downloading: {}", path.to_str().unwrap()).ok();
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
                    writeln!(&mut handle, "File already exists: {}", path.to_str().chain_err(|| UNABLE_TO_CONVERT_TO_STR)?).ok();
                }
            }
        }
        Ok(())
    }
}
