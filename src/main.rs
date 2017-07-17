extern crate rss;

mod structs;

use std::fs::File;
use std::io::BufReader;
use rss::{Channel, Item};
use structs::*;

fn main() {
    let file = File::open("rss.xml").unwrap();
    let channel = Channel::read_from(BufReader::new(file)).unwrap();
    let podcast = Podcast::from(channel);

    for title in podcast.list_titles() {
        println!("{}", title);
    }
    let ep = &podcast.episodes()[0];
    println!(
        "{}",
        match ep.download_url() {
            Some(val) => val,
            None => "",
        }
    );
}
