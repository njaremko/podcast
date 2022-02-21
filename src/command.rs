use crate::{executor, structs::State};
use anyhow::Result;
use clap::{ArgMatches, Command};

pub enum CommandC<'a> {
    Download(State, ArgMatches),
    List(State, ArgMatches),
    Play(State, ArgMatches),
    Subscribe(State, ArgMatches),
    Search(State, ArgMatches),
    Remove(State, ArgMatches),
    Complete(State, Command<'a>, ArgMatches),
    Refresh(State),
    Update(State),
    NoMatch(State),
}

pub fn parse_command<'a>(state: State, app: Command<'a>, matches: ArgMatches) -> CommandC<'a> {
    let state_copy = state.clone();
    matches
        .subcommand_name()
        .map(|command| match command {
            "download" => CommandC::Download(
                state,
                matches.subcommand_matches("download").unwrap().clone(),
            ),
            "ls" | "list" => CommandC::List(
                state,
                matches
                    .subcommand_matches("ls")
                    .or_else(|| matches.subcommand_matches("list"))
                    .unwrap()
                    .clone(),
            ),
            "play" => CommandC::Play(state, matches.subcommand_matches("play").unwrap().clone()),
            "sub" | "subscribe" => CommandC::Subscribe(
                state,
                matches
                    .subcommand_matches("sub")
                    .or_else(|| matches.subcommand_matches("subscribe"))
                    .unwrap()
                    .clone(),
            ),
            "search" => {
                CommandC::Search(state, matches.subcommand_matches("search").unwrap().clone())
            }
            "rm" => CommandC::Remove(state, matches.subcommand_matches("rm").unwrap().clone()),
            "completion" => CommandC::Complete(
                state,
                app,
                matches.subcommand_matches("completion").unwrap().clone(),
            ),
            "refresh" => CommandC::Refresh(state),
            "update" => CommandC::Update(state),
            _ => CommandC::NoMatch(state),
        })
        .unwrap_or_else(|| CommandC::NoMatch(state_copy))
}

pub async fn run_command<'a>(command: CommandC<'a>) -> Result<State> {
    match command {
        CommandC::Download(state, matches) => executor::download(state, &matches).await,
        CommandC::List(state, matches) => executor::list(state, &matches),
        CommandC::Play(state, matches) => executor::play(state, &matches),
        CommandC::Subscribe(state, matches) => executor::subscribe(state, &matches).await,
        CommandC::Search(state, matches) => executor::search(state, &matches).await,
        CommandC::Remove(state, matches) => executor::remove(state, &matches),
        CommandC::Complete(state, mut app, matches) => {
            executor::complete(&mut app, &matches)?;
            Ok(state)
        }
        CommandC::Refresh(mut state) => {
            state.update_rss().await?;
            Ok(state)
        }
        CommandC::Update(state) => {
            state.check_for_update().await?;
            Ok(state)
        }
        CommandC::NoMatch(state) => Ok(state),
    }
}
