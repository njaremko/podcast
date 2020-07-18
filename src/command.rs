use crate::{executor, structs::State};
use anyhow::Result;
use clap::{App, ArgMatches};

pub enum Command<'a> {
    Download(State, ArgMatches<'a>),
    List(State, ArgMatches<'a>),
    Play(State, ArgMatches<'a>),
    Subscribe(State, ArgMatches<'a>),
    Search(State, ArgMatches<'a>),
    Remove(State, ArgMatches<'a>),
    Complete(State, App<'a, 'a>, ArgMatches<'a>),
    Refresh(State),
    Update(State),
    NoMatch(State),
}

pub fn parse_command<'a>(state: State, app: App<'a, 'a>, matches: ArgMatches<'a>) -> Command<'a> {
    let state_copy = state.clone();
    matches
        .subcommand_name()
        .map(|command| match command {
            "download" => Command::Download(
                state,
                matches.subcommand_matches("download").unwrap().clone(),
            ),
            "ls" | "list" => Command::List(
                state,
                matches
                    .subcommand_matches("ls")
                    .or_else(|| matches.subcommand_matches("list"))
                    .unwrap()
                    .clone(),
            ),
            "play" => Command::Play(state, matches.subcommand_matches("play").unwrap().clone()),
            "sub" | "subscribe" => Command::Subscribe(
                state,
                matches
                    .subcommand_matches("sub")
                    .or_else(|| matches.subcommand_matches("subscribe"))
                    .unwrap()
                    .clone(),
            ),
            "search" => {
                Command::Search(state, matches.subcommand_matches("search").unwrap().clone())
            }
            "rm" => Command::Remove(state, matches.subcommand_matches("rm").unwrap().clone()),
            "completion" => Command::Complete(
                state,
                app,
                matches.subcommand_matches("completion").unwrap().clone(),
            ),
            "refresh" => Command::Refresh(state),
            "update" => Command::Update(state),
            _ => Command::NoMatch(state),
        })
        .unwrap_or_else(|| Command::NoMatch(state_copy))
}

pub async fn run_command<'a>(command: Command<'a>) -> Result<State> {
    match command {
        Command::Download(state, matches) => executor::download(state, &matches).await,
        Command::List(state, matches) => executor::list(state, &matches),
        Command::Play(state, matches) => executor::play(state, &matches),
        Command::Subscribe(state, matches) => executor::subscribe(state, &matches).await,
        Command::Search(state, matches) => executor::search(state, &matches).await,
        Command::Remove(state, matches) => executor::remove(state, &matches),
        Command::Complete(state, mut app, matches) => {
            executor::complete(&mut app, &matches)?;
            Ok(state)
        }
        Command::Refresh(mut state) => {
            state.update_rss().await?;
            Ok(state)
        }
        Command::Update(state) => {
            state.check_for_update().await?;
            Ok(state)
        }
        Command::NoMatch(state) => Ok(state),
    }
}
