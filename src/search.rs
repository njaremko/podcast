use failure::Error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest;

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

pub async fn search_for_podcast(podcast: &str) -> Result<Response, Error> {
    let encoded = utf8_percent_encode(podcast, NON_ALPHANUMERIC).to_string();
    let url = BASE.to_string() + "/query/" + &encoded;
    let r = reqwest::get(&url).await?.json::<Response>().await?;
    Ok(r)
}

pub async fn search_for_episode(podcast: &str, ep: &str) -> Result<Response, Error> {
    let podcast_encoded = utf8_percent_encode(podcast, NON_ALPHANUMERIC).to_string();
    let ep_encoded = utf8_percent_encode(ep, NON_ALPHANUMERIC).to_string();
    let url = BASE.to_string() + "/query/" + &podcast_encoded + "/episode/" + &ep_encoded;
    let r = reqwest::get(&url).await?.json::<Response>().await?;
    Ok(r)
}

pub async fn retrieve_rss(podcast: &str) -> Result<RssResponse, Error> {
    let encoded = utf8_percent_encode(podcast, NON_ALPHANUMERIC).to_string();
    let url = BASE.to_string() + "/rss/" + &encoded;
    let r = reqwest::get(&url).await?.json::<RssResponse>().await?;
    Ok(r)
}
