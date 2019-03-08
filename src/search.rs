use failure::Error;
use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use reqwest;
use serde_json;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    error: Option<String>,
    found: Vec<String>,
}

impl Response {
    pub fn error(&self) -> &Option<String> {
        &self.error
    }

    pub fn found(&self) -> &[String] {
        &self.found
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RssResponse {
    error: Option<String>,
    url: Option<String>,
}

impl RssResponse {
    pub fn error(&self) -> &Option<String> {
        &self.error
    }

    pub fn url(&self) -> &Option<String> {
        &self.url
    }
}

const BASE: &str = "https://podcastapi.ca";

pub fn search_for_podcast(podcast: &str) -> Result<Response, Error> {
    let encoded = utf8_percent_encode(podcast, DEFAULT_ENCODE_SET).to_string();
    let url = BASE.to_string() + "/query/" + &encoded;
    let resp = reqwest::get(&url)?;
    let r: Response = serde_json::from_reader(resp)?;
    Ok(r)
}

pub fn search_for_episode(podcast: &str, ep: &str) -> Result<Response, Error> {
    let podcast_encoded = utf8_percent_encode(podcast, DEFAULT_ENCODE_SET).to_string();
    let ep_encoded = utf8_percent_encode(ep, DEFAULT_ENCODE_SET).to_string();
    let url = BASE.to_string() + "/query/" + &podcast_encoded + "/episode/" + &ep_encoded;
    let resp = reqwest::get(&url)?;
    let r: Response = serde_json::from_reader(resp)?;
    Ok(r)
}

pub fn retrieve_rss(podcast: &str) -> Result<RssResponse, Error> {
    let encoded = utf8_percent_encode(podcast, DEFAULT_ENCODE_SET).to_string();
    let url = BASE.to_string() + "/rss/" + &encoded;
    let resp = reqwest::get(&url)?;
    let r: RssResponse = serde_json::from_reader(resp)?;
    Ok(r)
}
