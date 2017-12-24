use std::collections::HashSet;
use std::env;
use std::fs::{self, DirBuilder, File};
use std::io::{self, BufReader, Read, Write};
use std::num::ParseIntError;
use std::path::PathBuf;

use reqwest;
use rss::Channel;

pub fn trim_extension(filename: &str) -> Option<String> {
    let name = String::from(filename);
    match name.rfind('.') {
        Some(index) => Some(String::from(&name[0..index])),
        None => None,
    }
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

pub fn create_directories() -> Result<(), String> {
    let mut path = get_podcast_dir();
    path.push(".rss");
    if let Err(err) = DirBuilder::new().recursive(true).create(&path) {
        return Err(format!(
            "Couldn't create directory: {}\nReason: {}",
            path.to_str().unwrap(),
            err
        ));
    }
    Ok(())
}

pub fn already_downloaded(dir: &str) -> Result<HashSet<String>, io::Error> {
    let mut result = HashSet::new();

    let mut path = get_podcast_dir();
    path.push(dir);

    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        match entry.file_name().into_string() {
            Ok(val) => {
                let name = String::from(val);
                let index = name.find('.').unwrap();
                result.insert(String::from(&name[0..index]));
            }
            Err(_) => {
                println!(
                    "OsString: {:?} couldn't be converted to String",
                    entry.file_name()
                );
            }
        }
    }
    Ok(result)
}

pub fn get_podcast_dir() -> PathBuf {
    match env::var_os("PODCAST") {
        Some(val) => PathBuf::from(val),
        None => {
            let mut path = env::home_dir().expect("Couldn't find your home directory");
            path.push("Podcasts");
            path
        }
    }
}

pub fn get_sub_file() -> PathBuf {
    let mut path = get_podcast_dir();
    path.push(".subscriptions");
    path
}

pub fn get_xml_dir() -> PathBuf {
    let mut path = get_podcast_dir();
    path.push(".rss");
    path
}

pub fn download_rss_feed(url: &str) -> Result<Channel, String> {
    let mut path = get_podcast_dir();
    path.push(".rss");
    DirBuilder::new().recursive(true).create(&path).unwrap();
    let mut resp = reqwest::get(url).unwrap();
    let mut content: Vec<u8> = Vec::new();
    resp.read_to_end(&mut content).unwrap();
    let channel = Channel::read_from(BufReader::new(&content[..])).unwrap();
    let mut filename = String::from(channel.title());
    filename.push_str(".xml");
    path.push(filename);
    let mut file = File::create(&path).unwrap();
    file.write_all(&content).unwrap();
    Ok(channel)
}

pub fn parse_download_episodes(e_search: &str) -> Result<Vec<usize>, ParseIntError> {
    let input = String::from(e_search);
    let mut ranges = Vec::<(usize, usize)>::new();
    let mut elements = Vec::<usize>::new();
    let comma_separated: Vec<&str> = input.split(',').collect();
    for elem in comma_separated {
        let temp = String::from(elem);
        if temp.contains("-") {
            let range: Vec<usize> = elem.split('-')
                .map(|i| i.parse::<usize>().unwrap())
                .collect();
            ranges.push((range[0], range[1]));
        } else {
            elements.push(elem.parse::<usize>()?);
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
