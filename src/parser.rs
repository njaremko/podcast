use clap::{App, Arg, SubCommand};

pub fn get_app<'a, 'b>(version: &'a str) -> App<'a, 'b> {
    App::new("podcast")
        .version(version)
        .author("Nathan J. <njaremko@gmail.com>")
        .about("A command line podcast manager")
        .subcommand(
            SubCommand::with_name("download")
                .about("download episodes of podcast")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("EPISODE")
                        .required(false)
                        .help("Episode index")
                        .index(2),
                )
                .arg(
                    Arg::with_name("latest")
                    .short("l")
                    .long("latest")
                    .value_name("LATEST")
                    .help("Downloads this many of the latest episodes")
                    .takes_value(true)
                    .required(false),
                )
                .arg(
                    Arg::with_name("name")
                        .short("e")
                        .long("episode")
                        .help("Download using episode name instead of index number")
                        .required(false),
                )
                .arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("Download all matching episodes")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("ls")
                .about("list episodes of podcast")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("list episodes of podcast")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("play")
                .about("play an episode")
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("EPISODE")
                        .help("Episode index")
                        .required(false)
                        .index(2),
                )
                .arg(
                    Arg::with_name("name")
                        .short("e")
                        .long("episode")
                        .help("Play using episode name instead of index number")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("search")
                .about("searches for podcasts")
                .arg(
                    Arg::with_name("debug")
                        .short("d")
                        .help("print debug information verbosely"),
                )
                .arg(
                    Arg::with_name("PODCAST")
                        .help("Regex for subscribed podcast")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("EPISODE")
                        .help("Episode index")
                        .required(false)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("subscribe")
                .about("subscribes to a podcast RSS feed")
                .arg(
                    Arg::with_name("URL")
                        .help("URL to RSS feed")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("sub")
                .about("subscribes to a podcast RSS feed")
                .arg(
                    Arg::with_name("URL")
                        .help("URL to RSS feed")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("refresh").about("refresh subscribed podcasts"))
        .subcommand(SubCommand::with_name("update").about("check for updates"))
        .subcommand(
            SubCommand::with_name("rm")
                .about("unsubscribe from a podcast")
                .arg(Arg::with_name("PODCAST").help("Podcast to delete").index(1)),
        )
        .subcommand(
            SubCommand::with_name("completion")
                .about("install shell completion")
                .arg(
                    Arg::with_name("SHELL")
                        .help("Shell to print completion for")
                        .index(1),
                ),
        )
}
