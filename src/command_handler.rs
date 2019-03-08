use clap::{App, ArgMatches};

use crate::actions::*;
use crate::arg_parser;
use crate::commands;
use crate::errors::*;
use crate::structs::*;

pub fn parse_sub_command(matches: &ArgMatches) -> commands::Command {
    match matches.subcommand_name() {
        Some("download") => commands::Command::Download,
        Some("ls") | Some("list") => commands::Command::List,
        Some("play") => commands::Command::Play,
        Some("sub") | Some("subscribe") => commands::Command::Subscribe,
        Some("search") => commands::Command::Search,
        Some("rm") => commands::Command::Remove,
        Some("completion") => commands::Command::Complete,
        Some("refresh") => commands::Command::Refresh,
        Some("update") => commands::Command::Update,
        _ => commands::Command::NoMatch,
    }
}

pub fn handle_matches(
    version: &str,
    state: &mut State,
    config: Config,
    app: &mut App,
    matches: &ArgMatches,
) -> Result<()> {
    let command = parse_sub_command(matches);
    match command {
        commands::Command::Download => {
            arg_parser::download(state, matches)?;
        }
        commands::Command::List => {
            arg_parser::list(state, matches)?;
        }
        commands::Command::Play => {
            arg_parser::play(state, matches)?;
        }
        commands::Command::Subscribe => {
            arg_parser::subscribe(state, config, matches)?;
        }
        commands::Command::Search => {
            arg_parser::search(state, config, matches)?;
        }
        commands::Command::Remove => {
            arg_parser::remove(state, matches)?;
        }
        commands::Command::Complete => {
            arg_parser::complete(app, matches)?;
        }
        commands::Command::Refresh => {
            update_rss(state);
        }
        commands::Command::Update => {
            check_for_update(version)?;
        }
        _ => (),
    };
    Ok(())
}
