use std::fs::File;
use std::io::BufReader;
use rss::{Channel, Item};

pub struct Podcast(Channel);

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
    pub fn episodes(&self) -> Vec<Episode> {
        let mut result = Vec::new();

        let items = self.0.items().to_vec();
        for item in items {
            result.push(Episode::from(item));
        }
        result
    }


    pub fn list_titles(&self) -> Vec<&str> {
        let mut result = Vec::new();

        let items = self.0.items();
        for item in items {
            match item.title() {
                Some(val) => result.push(val),
                None => (),
            }
        }
        result
    }
}

impl Episode {
    pub fn download_url(&self) -> Option<&str> {
        match self.0.enclosure() {
            Some(val) => Some(val.url()),
            None => None, 
        }
    }
}
