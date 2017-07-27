use std::collections::HashSet;
use std::env;
use std::fs::DirBuilder;
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;

pub fn trim_extension(filename: &str) -> String {
    let name = String::from(filename);
    let index = name.rfind('.').unwrap();
    String::from(&name[0..index])
}

pub fn find_extension(input: &str) -> Option<&str> {
    let tmp = String::from(input);
    if tmp.contains(".mp3") {
        Some(".mp3")
    } else if tmp.contains(".m4a") {
        Some(".m4a")
    } else if tmp.contains(".wav") {
        Some(".wav")
    } else if tmp.contains(".ogg") {
        Some(".ogg")
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

pub fn already_downloaded(dir: &str) -> HashSet<String> {
    let mut result = HashSet::new();

    let mut path = get_podcast_dir();
    path.push(dir);

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
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
        }
    }
    result
}

pub fn get_podcast_dir() -> PathBuf {
    match env::var_os("PODCAST") {
        Some(val) => PathBuf::from(val),
        None => {
            let mut path = env::home_dir().unwrap();
            path.push("Podcasts");
            path
        }
    }
}

pub fn get_sub_file() -> PathBuf {
    match env::var_os("PODCAST") {
        Some(val) => {
            let mut path = PathBuf::from(val);
            path.push(".subscriptions");
            path
        }
        None => {
            let mut path = env::home_dir().unwrap();
            path.push("Podcasts");
            path.push(".subscriptions");
            path
        }
    }
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
