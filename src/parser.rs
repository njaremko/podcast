use clap::{Arg, Command};

pub fn get_app<'a, 'b>(version: &'a str) -> Command {
    Command::new("podcast")
        .version(version)
        .author("Nathan J. <njaremko@gmail.com>")
        .about("A command line podcast manager")
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Output less stuff")
                .required(false),
        )
        .subcommand(
            Command::new("download")
                .about("download episodes of podcast")
                .arg(
                    Arg::new("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("EPISODE")
                        .required(false)
                        .help("Episode index")
                        .index(2),
                )
                .arg(
                    Arg::new("latest")
                        .short('l')
                        .long("latest")
                        .value_name("LATEST")
                        .help("Downloads this many of the latest episodes")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::new("name")
                        .short('e')
                        .long("episode")
                        .help("Download using episode name instead of index number")
                        .required(false),
                )
                .arg(
                    Arg::new("all")
                        .short('a')
                        .long("all")
                        .help("Download all matching episodes")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("ls").about("list episodes of podcast").arg(
                Arg::new("PODCAST")
                    .help("Regex for subscribed podcast")
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("list").about("list episodes of podcast").arg(
                Arg::new("PODCAST")
                    .help("Regex for subscribed podcast")
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("play")
                .about("play an episode")
                .arg(
                    Arg::new("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("EPISODE")
                        .help("Episode index")
                        .required(false)
                        .index(2),
                )
                .arg(
                    Arg::new("name")
                        .short('e')
                        .long("episode")
                        .help("Play using episode name instead of index number")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("search")
                .about("searches for podcasts")
                .arg(
                    Arg::new("debug")
                        .short('d')
                        .help("print debug information verbosely"),
                )
                .arg(
                    Arg::new("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1)
                        .multiple_occurrences(true),
                ),
        )
        .subcommand(
            Command::new("subscribe")
                .about("subscribes to a podcast RSS feed")
                .arg(
                    Arg::new("URL")
                        .help("URL to RSS feed")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("sub")
                .about("subscribes to a podcast RSS feed")
                .arg(
                    Arg::new("URL")
                        .help("URL to RSS feed")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("refresh").about("refresh subscribed podcasts"))
        .subcommand(Command::new("update").about("check for updates"))
        .subcommand(
            Command::new("rm")
                .about("unsubscribe from a podcast")
                .arg(Arg::new("PODCAST").help("Podcast to delete").index(1)),
        )
        .subcommand(
            Command::new("completion")
                .about("install shell completion")
                .arg(
                    Arg::new("SHELL")
                        .help("Shell to print completion for")
                        .index(1),
                ),
        )
}
