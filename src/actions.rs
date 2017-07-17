use regex::Regex;
use structs::*;

pub fn list_episodes(state: State, search: &str) {
    let re = Regex::new(&search).unwrap();
    for podcast in state.subscriptions() {
        if re.is_match(&podcast.name) {
            println!("Episodes for {}:", &podcast.name);
            match Podcast::from_url(&podcast.url) {
                Ok(podcast) => {
                    for title in podcast.list_episodes() {
                        println!("{}", title)
                    }
                }
                Err(err) => println!("{}", err),
            }

        }
    }
}

pub fn list_subscriptions(state: State) {
    for podcast in state.subscriptions() {
        println!("{}", podcast.name);
    }
}
