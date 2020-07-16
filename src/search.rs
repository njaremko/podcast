use failure::Error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistSearchResponse {
    #[serde(rename = "resultCount")]
    pub result_count: usize,
    pub results: Vec<ArtistSearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistSearchResult {
    #[serde(rename = "wrapperType")]
    pub wrapper_type: Option<String>,
    #[serde(rename = "artistType")]
    pub artist_type: Option<String>,
    #[serde(rename = "artistName")]
    pub artist_name: Option<String>,
    #[serde(rename = "artistLinkUrl")]
    pub artist_link_url: Option<String>,
    #[serde(rename = "artistId")]
    pub artist_id: Option<i64>,
    #[serde(rename = "primaryGenreName")]
    pub primary_genre_name: Option<String>,
    #[serde(rename = "primaryGenreId")]
    pub primary_genre_id: Option<i64>,
}



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PodcastSearchResult {
    #[serde(rename = "wrapperType")]
    wrapper_type: Option<WrapperType>,
    kind: Option<Kind>,
    #[serde(rename = "artistId")]
    artist_id: Option<i64>,
    #[serde(rename = "collectionId")]
    collection_id: Option<i64>,
    #[serde(rename = "trackId")]
    track_id: Option<i64>,
    #[serde(rename = "artistName")]
    artist_name: Option<String>,
    #[serde(rename = "collectionName")]
    pub collection_name: Option<String>,
    #[serde(rename = "trackName")]
    track_name: Option<String>,
    #[serde(rename = "collectionCensoredName")]
    collection_censored_name: Option<String>,
    #[serde(rename = "trackCensoredName")]
    track_censored_name: Option<String>,
    #[serde(rename = "artistViewUrl")]
    artist_view_url: Option<String>,
    #[serde(rename = "collectionViewUrl")]
    collection_view_url: Option<String>,
    #[serde(rename = "feedUrl")]
    pub feed_url: Option<String>,
    #[serde(rename = "trackViewUrl")]
    track_view_url: Option<String>,
    #[serde(rename = "artworkUrl30")]
    artwork_url30: Option<String>,
    #[serde(rename = "artworkUrl60")]
    artwork_url60: Option<String>,
    #[serde(rename = "artworkUrl100")]
    artwork_url100: Option<String>,
    #[serde(rename = "collectionPrice")]
    collection_price: Option<f64>,
    #[serde(rename = "trackPrice")]
    track_price: Option<f64>,
    #[serde(rename = "trackRentalPrice")]
    track_rental_price: Option<f64>,
    #[serde(rename = "collectionHdPrice")]
    collection_hd_price: Option<f64>,
    #[serde(rename = "trackHdPrice")]
    track_hd_price: Option<f64>,
    #[serde(rename = "trackHdRentalPrice")]
    track_hd_rental_price: Option<f64>,
    #[serde(rename = "releaseDate")]
    release_date: Option<String>,
    #[serde(rename = "collectionExplicitness")]
    collection_explicitness: Option<Explicitness>,
    #[serde(rename = "trackExplicitness")]
    track_explicitness: Option<Explicitness>,
    #[serde(rename = "trackCount")]
    track_count: Option<i64>,
    country: Option<Country>,
    currency: Option<Currency>,
    #[serde(rename = "primaryGenreName")]
    primary_genre_name: Option<String>,
    #[serde(rename = "contentAdvisoryRating")]
    content_advisory_rating: Option<ContentAdvisoryRating>,
    #[serde(rename = "artworkUrl600")]
    artwork_url600: Option<String>,
    #[serde(rename = "genreIds")]
    genre_ids: Option<Vec<String>>,
    genres: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Explicitness {
    #[serde(rename = "cleaned")]
    Cleaned,
    #[serde(rename = "explicit")]
    Explicit,
    #[serde(rename = "notExplicit")]
    NotExplicit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContentAdvisoryRating {
    Clean,
    Explicit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Country {
    #[serde(rename = "USA")]
    Usa,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Currency {
    #[serde(rename = "USD")]
    Usd,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    #[serde(rename = "podcast")]
    Podcast,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WrapperType {
    #[serde(rename = "track")]
    Track,
}



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrackSearchResponse {
    #[serde(rename = "resultCount")]
    pub result_count: usize,
    pub results: Vec<PodcastSearchResult>
}

impl TrackSearchResponse {
    pub fn result_count(&self) -> usize {
        self.result_count
    }

    pub fn results(&self) -> &[PodcastSearchResult] {
        &self.results
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

const BASE: &str = "https://itunes.apple.com/search?media=podcast&term=";

pub async fn search_for_podcast(podcast: &str) -> Result<TrackSearchResponse, Error> {
    let encoded: String = podcast.chars().map(|c| {
        if c == ' ' {
            return '+';
        }
        return c;
    }).collect();
    let url =  format!("https://itunes.apple.com/search?media=podcast&entity=podcast&term={}", encoded);
    let r = reqwest::get(&url).await?.json::<TrackSearchResponse>().await?;
    Ok(r)
}

pub async fn search_for_episode(podcast: &str, ep: &str) -> Result<TrackSearchResponse, Error> {
    let podcast_encoded = utf8_percent_encode(podcast, NON_ALPHANUMERIC).to_string();
    let ep_encoded = utf8_percent_encode(ep, NON_ALPHANUMERIC).to_string();
    let url = BASE.to_string() + "/query/" + &podcast_encoded + "/episode/" + &ep_encoded;
    let r = reqwest::get(&url).await?.json::<TrackSearchResponse>().await?;
    Ok(r)
}

pub async fn retrieve_rss(podcast: &str) -> Result<TrackSearchResponse, Error> {
    let encoded = utf8_percent_encode(podcast, NON_ALPHANUMERIC).to_string();
    let url =  format!("https://itunes.apple.com/lookup?id={}&entity=podcast", encoded);
    let r = reqwest::get(&url).await?.json::<TrackSearchResponse>().await?;
    Ok(r)
}
