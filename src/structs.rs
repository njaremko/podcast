use actions::*;
use chrono::prelude::*;
use rayon::prelude::*;
use reqwest;
use rss::{self, Channel, Item};
use serde_json;
use std::collections::BTreeSet;
use std::fs::{DirBuilder, File};
use std::io::{self, Read, Write};
use utils::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Subscription {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    last_run_time: DateTime<Utc>,
    subs: Vec<Subscription>,
}

impl State {
    pub fn new() -> State {
        let mut path = get_podcast_dir();
        path.push(".subscriptions");
        if path.exists() {
            let mut s = String::new();
            File::open(&path).unwrap().read_to_string(&mut s).unwrap();
            let mut state: State = serde_json::from_str(&s).unwrap();
            // Check if a day has passed (86400 seconds)
            if state
                .last_run_time
                .signed_duration_since(Utc::now())
                .num_seconds() < -86400
            {
                update_rss(&state.clone());
            }
            state.last_run_time = Utc::now();
            state
        } else {
            State {
                last_run_time: Utc::now(),
                subs: Vec::new(),
            }
        }
    }

    pub fn subscribe(&mut self, url: &str) {
        let mut set = BTreeSet::new();
        for sub in self.subscriptions() {
            set.insert(sub.url);
        }
        if !set.contains(url) {
            let channel = Channel::from_url(url).unwrap();
            self.subs.push(Subscription {
                name: String::from(channel.title()),
                url: String::from(url),
            });
        }
        if let Err(err) = self.save() {
            println!("{}", err);
        }
        // TODO only download new rss, don't refresh all
        update_rss(&self.clone());
    }

    pub fn subscriptions(&self) -> Vec<Subscription> {
        self.subs.clone()
    }

    pub fn save(&self) -> Result<(), io::Error> {
        // TODO write to a temp file and rename instead of overwriting

        let mut path = get_podcast_dir();
        DirBuilder::new().recursive(true).create(&path).unwrap();
        path.push(".subscriptions");
        let serialized = serde_json::to_string(self)?;
        let mut file = File::create(&path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Podcast(Channel);

#[derive(Clone)]
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

    pub fn from_url(url: &str) -> Result<Podcast, rss::Error> {
        match Channel::from_url(url) {
            Ok(val) => Ok(Podcast::from(val)),
            Err(err) => Err(err),
        }
    }

    pub fn episodes(&self) -> Vec<Episode> {
        let mut result = Vec::new();

        let items = self.0.items().to_vec();
        for item in items {
            result.push(Episode::from(item));
        }
        result
    }

    pub fn download(&self) {
        let mut path = get_podcast_dir();
        path.push(self.title());

        let downloaded = already_downloaded(self.title());

        self.episodes().par_iter().for_each(
            |ref i| if let Some(ep_title) =
                i.title()
            {
                if !downloaded.contains(ep_title) {
                    if let Err(err) = i.download(self.title()) {
                        println!("{}", err);
                    }
                }
            },
        );
    }

    pub fn download_specific(&self, episode_numbers: Vec<usize>) {
        let mut path = get_podcast_dir();
        path.push(self.title());

        let downloaded = already_downloaded(self.title());
        let episodes = self.episodes();

        episode_numbers.par_iter().for_each(
            |ep_num| if let Some(ep_title) =
                episodes[episodes.len() - ep_num].title()
            {
                if !downloaded.contains(ep_title) {
                    if let Err(err) = episodes[episodes.len() - ep_num].download(self.title()) {
                        println!("{}", err);
                    }
                }
            },
        );
    }
}

impl Episode {
    pub fn title(&self) -> Option<&str> {
        self.0.title()
    }

    pub fn url(&self) -> Option<&str> {
        match self.0.enclosure() {
            Some(val) => Some(val.url()),
            None => None,
        }
    }

    pub fn extension(&self) -> Option<&str> {
        match self.0.enclosure() {
            Some(enclosure) => {
                match enclosure.mime_type() {
                    "audio/mpeg" => Some(".mp3"),
                    "audio/mp4" => Some(".m4a"),
                    "audio/ogg" => Some(".ogg"),
                    _ => None,
                }
            }
            None => None,
        }
    }

    pub fn download(&self, podcast_name: &str) -> Result<(), io::Error> {
        let mut path = get_podcast_dir();
        path.push(podcast_name);
        DirBuilder::new().recursive(true).create(&path).unwrap();

        if let Some(url) = self.url() {
            if let Some(title) = self.title() {
                println!("Downloading: {}", title);
                let mut filename = String::from(title);
                filename.push_str(self.extension().unwrap());
                path.push(filename);
                let mut file = File::create(&path)?;
                let mut resp = reqwest::get(url).unwrap();
                let mut content: Vec<u8> = Vec::new();
                resp.read_to_end(&mut content)?;
                file.write_all(&content)?;
                return Ok(());
            }
        }
        Ok(())
    }
}
