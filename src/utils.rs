use std::collections::HashSet;
use std::env;
use std::fs::{self, DirBuilder, File};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

use errors::*;
use reqwest;
use rss::Channel;

pub const UNABLE_TO_PARSE_REGEX: &'static str = "unable to parse regex";
pub const UNABLE_TO_OPEN_FILE: &'static str = "unable to open file";
pub const UNABLE_TO_CREATE_FILE: &'static str = "unable to create file";
pub const UNABLE_TO_WRITE_FILE: &'static str = "unable to write file";
pub const UNABLE_TO_READ_FILE_TO_STRING: &'static str = "unable to read file to string";
pub const UNABLE_TO_READ_DIRECTORY: &'static str = "unable to read directory";
pub const UNABLE_TO_READ_ENTRY: &'static str = "unable to read entry";
pub const UNABLE_TO_CREATE_DIRECTORY: &'static str = "unable to create directory";
pub const UNABLE_TO_READ_RESPONSE_TO_END: &'static str = "unable to read response to end";
pub const UNABLE_TO_GET_HTTP_RESPONSE: &'static str = "unable to get http response";
pub const UNABLE_TO_CONVERT_TO_STR: &'static str = "unable to convert to &str";
pub const UNABLE_TO_REMOVE_FILE: &'static str = "unable to remove file";
pub const UNABLE_TO_CREATE_CHANNEL_FROM_RESPONSE: &'static str =
    "unable to create channel from http response";
pub const UNABLE_TO_CREATE_CHANNEL_FROM_FILE: &'static str =
    "unable to create channel from xml file";
pub const UNABLE_TO_RETRIEVE_PODCAST_BY_TITLE: &'static str = "unable to retrieve podcast by title";
pub fn trim_extension(filename: &str) -> Option<String> {
    let name = String::from(filename);
    let index = name.rfind('.')?;
    Some(String::from(&name[0..index]))
}

pub fn find_extension(input: &str) -> Option<&str> {
    let tmp = String::from(input);
    if tmp.ends_with(".mp3") {
        Some(".mp3")
    } else if tmp.ends_with(".m4a") {
        Some(".m4a")
    } else if tmp.ends_with(".wav") {
        Some(".wav")
    } else if tmp.ends_with(".ogg") {
        Some(".ogg")
    } else if tmp.ends_with(".opus") {
        Some(".opus")
    } else {
        None
    }
}

pub fn get_podcast_dir() -> Result<PathBuf> {
    match env::var_os("PODCAST") {
        Some(val) => Ok(PathBuf::from(val)),
        None => {
            let mut path = env::home_dir().chain_err(|| "Couldn't find your home directory")?;
            path.push("Podcasts");
            Ok(path)
        }
    }
}

pub fn create_directories() -> Result<()> {
    let mut path = get_podcast_dir()?;
    path.push(".rss");
    DirBuilder::new()
        .recursive(true)
        .create(&path)
        .chain_err(|| "unable to create directory")
}

pub fn already_downloaded(dir: &str) -> Result<HashSet<String>> {
    let mut result = HashSet::new();

    let mut path = get_podcast_dir()?;
    path.push(dir);

    let entries = fs::read_dir(path).chain_err(|| "unable to read directory")?;
    for entry in entries {
        let entry = entry.chain_err(|| "unable to read entry")?;
        match entry.file_name().into_string() {
            Ok(name) => {
                let index = name.find('.').chain_err(|| "unable to find string index")?;
                result.insert(String::from(&name[0..index]));
            }
            Err(_) => {
                eprintln!(
                    "OsString: {:?} couldn't be converted to String",
                    entry.file_name()
                );
            }
        }
    }
    Ok(result)
}

pub fn get_sub_file() -> Result<PathBuf> {
    let mut path = get_podcast_dir()?;
    path.push(".subscriptions");
    Ok(path)
}

pub fn get_xml_dir() -> Result<PathBuf> {
    let mut path = get_podcast_dir()?;
    path.push(".rss");
    Ok(path)
}

pub fn download_rss_feed(url: &str) -> Result<Channel> {
    let mut path = get_podcast_dir()?;
    path.push(".rss");
    DirBuilder::new()
        .recursive(true)
        .create(&path)
        .chain_err(|| "unable to open directory")?;
    let mut resp = reqwest::get(url).chain_err(|| "unable to open url")?;
    let mut content: Vec<u8> = Vec::new();
    resp.read_to_end(&mut content)
        .chain_err(|| "unable to read http response to end")?;
    let channel = Channel::read_from(BufReader::new(&content[..]))
        .chain_err(|| "unable to create channel from xml http response")?;
    let mut filename = String::from(channel.title());
    filename.push_str(".xml");
    path.push(filename);
    let mut file = File::create(&path).chain_err(|| "unable to create file")?;
    file.write_all(&content)
        .chain_err(|| "unable to write file")?;
    Ok(channel)
}

pub fn parse_download_episodes(e_search: &str) -> Result<Vec<usize>> {
    let input = String::from(e_search);
    let mut ranges = Vec::<(usize, usize)>::new();
    let mut elements = Vec::<usize>::new();
    let comma_separated: Vec<&str> = input.split(',').collect();
    for elem in comma_separated {
        let temp = String::from(elem);
        if temp.contains('-') {
            let range: Vec<usize> = elem.split('-')
                .map(|i| i.parse::<usize>().chain_err(|| "unable to parse number"))
                .collect::<Result<Vec<usize>>>()
                .chain_err(|| "unable to collect ranges")?;
            ranges.push((range[0], range[1]));
        } else {
            elements.push(elem.parse::<usize>()
                .chain_err(|| "unable to parse number")?);
        }
    }

    for range in ranges {
        // Add 1 to upper range to include given episode in the download
        for num in range.0..range.1 + 1 {
            elements.push(num);
        }
    }
    elements.dedup();
    Ok(elements)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_extension_mp3() {
        assert_eq!(find_extension("test.mp3"), Some(".mp3"))
    }

    #[test]
    fn test_find_extension_m4a() {
        assert_eq!(find_extension("test.m4a"), Some(".m4a"))
    }

    #[test]
    fn test_find_extension_wav() {
        assert_eq!(find_extension("test.wav"), Some(".wav"))
    }

    #[test]
    fn test_find_extension_ogg() {
        assert_eq!(find_extension("test.ogg"), Some(".ogg"))
    }

    #[test]
    fn test_find_extension_opus() {
        assert_eq!(find_extension("test.opus"), Some(".opus"))
    }

    #[test]
    fn test_find_extension_invalid() {
        assert_eq!(find_extension("test.taco"), None)
    }

    #[test]
    fn test_trim_extension() {
        assert_eq!(trim_extension("test.taco"), Some(String::from("test")))
    }

    #[test]
    fn test_trim_extension_invalid() {
        assert_eq!(trim_extension("test"), None)
    }
}
