use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;


pub fn already_downloaded(dir: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();

    let mut path = get_podcast_dir();
    path.push(dir);

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                match entry.file_name().into_string() {
                    Ok(val) => {
                        // TODO There has to be a better way to do this...later
                        result.insert(String::from(
                            val.trim_right_matches(".mp3")
                                .trim_right_matches(".m4a")
                                .trim_right_matches(".ogg"),
                        ));
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

pub fn parse_download_episodes(e_search: &str) -> Vec<usize> {
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
            elements.push(elem.parse::<usize>().unwrap());
        }
    }

    for range in ranges {
        // Add 1 to upper range to include given episode in the download
        for num in range.0..range.1 + 1 {
            elements.push(num);
        }
    }
    elements.dedup();
    elements
}
