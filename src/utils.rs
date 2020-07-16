use std::collections::HashSet;
use std::env;
use std::fs::{self, DirBuilder, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

use anyhow::Result;
use dirs;
use reqwest;
use rss::Channel;

const UNSUBSCRIBE_NOTE: &str = "Note: this does NOT delete any downloaded podcasts";

pub fn trim_extension(filename: &str) -> Option<String> {
    let name = String::from(filename);
    if name.contains('.') {
        name.rfind('.').map(|index| String::from(&name[0..index]))
    } else {
        Some(name)
    }
}

pub fn find_extension(input: &str) -> Option<String> {
    let s: Vec<String> = input
        .split('.')
        .map(std::string::ToString::to_string)
        .collect();
    if s.len() > 1 {
        return s.last().cloned();
    }
    None
}

pub fn get_podcast_dir() -> Result<PathBuf> {
    match env::var_os("PODCAST") {
        Some(val) => Ok(PathBuf::from(val)),
        None => {
            let mut path = dirs::home_dir().unwrap();
            path.push("Podcasts");
            Ok(path)
        }
    }
}

pub fn append_extension(filename: &str, ext: &str) -> String {
    let mut f = filename.to_string();
    if !f.ends_with('.') {
        f.push_str(".");
    }
    f.push_str(ext);
    f
}

pub fn create_dir_if_not_exist(path: &PathBuf) -> Result<()> {
    DirBuilder::new().recursive(true).create(&path)?;
    Ok(())
}

pub fn create_directories() -> Result<()> {
    let mut path = get_podcast_dir()?;
    path.push(".rss");
    create_dir_if_not_exist(&path)
}

pub fn delete(title: &str) -> Result<()> {
    let mut path = get_xml_dir()?;
    let mut filename = String::from(title);
    filename.push_str(".xml");
    path.push(filename);
    println!("Removing '{}' from subscriptions...", &title);
    println!("{}", UNSUBSCRIBE_NOTE);
    fs::remove_file(path)?;
    Ok(())
}

pub fn delete_all() -> Result<()> {
    println!("Removing all subscriptions...");
    println!("{}", UNSUBSCRIBE_NOTE);
    fs::remove_dir_all(get_xml_dir()?)?;
    Ok(())
}

pub fn already_downloaded(dir: &str) -> Result<HashSet<String>> {
    let mut result = HashSet::new();

    let mut path = get_podcast_dir()?;
    path.push(dir);

    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        match entry.file_name().into_string() {
            Ok(name) => {
                let index = name.find('.').unwrap();
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
    path.push(".subscriptions.json");
    Ok(path)
}

pub fn get_xml_dir() -> Result<PathBuf> {
    let mut path = get_podcast_dir()?;
    path.push(".rss");
    Ok(path)
}

pub async fn download_rss_feed(url: &str) -> Result<Channel> {
    println!("Downloading RSS feed...");
    let mut path = get_podcast_dir()?;
    path.push(".rss");
    create_dir_if_not_exist(&path)?;
    let resp = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(BufReader::new(&resp[..]))?;
    let mut filename = String::from(channel.title());
    filename.push_str(".xml");
    path.push(filename);
    let mut file = File::create(&path)?;
    file.write_all(&resp)?;
    Ok(channel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_extension_mp3() {
        assert_eq!(find_extension("test.mp3"), Some("mp3".into()))
    }

    #[test]
    fn test_find_extension_m4a() {
        assert_eq!(find_extension("test.m4a"), Some("m4a".into()))
    }

    #[test]
    fn test_find_extension_wav() {
        assert_eq!(find_extension("test.wav"), Some("wav".into()))
    }

    #[test]
    fn test_find_extension_ogg() {
        assert_eq!(find_extension("test.ogg"), Some("ogg".into()))
    }

    #[test]
    fn test_find_extension_opus() {
        assert_eq!(find_extension("test.opus"), Some("opus".into()))
    }

    #[test]
    fn test_find_weird_extension() {
        assert_eq!(find_extension("test.taco"), Some("taco".into()))
    }

    #[test]
    fn test_find_no_extension() {
        assert_eq!(find_extension("test"), None)
    }

    #[test]
    fn test_trim_extension() {
        assert_eq!(trim_extension("test.taco"), Some(String::from("test")))
    }

    #[test]
    fn test_trim_extension_invalid() {
        assert_eq!(trim_extension("test"), Some("test".into()))
    }
}
