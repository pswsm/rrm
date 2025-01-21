extern crate core;

use crate::args::Options;
use clap::CommandFactory;
use rrm_locals::GamePath;
use std::{collections::HashSet, process::ExitCode};

mod args;
mod async_installer;
mod install;
mod list;
mod logger;
mod pull;
mod rimworldmod;
mod search;
mod utils;
use clap_complete::{Shell, generate};

#[cfg(feature = "dhat")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> ExitCode {
    #[cfg(feature = "dhat")]
    let _profiler = dhat::Profiler::new_heap();

    if app().is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[tokio::main]
async fn app() -> Result<(), ExitCode> {
    println!("Welcome!!!");
    let args: args::App = args::App::load();

    println!("Args loaded...");

    let installer = match rrm_installer::Installer::try_from_config() {
        Ok(installer) => installer,
        Err(reason) => {
            log!(Error: reason);
            std::process::exit(1)
        }
    };

    println!("Installer created OK");

    match args.command {
        args::Commands::Completions { shell } => {
            let mut matches = args::App::command();
            let shell = match shell.as_str() {
                "bash" => Shell::Bash,
                "fish" => Shell::Fish,
                "zsh" => Shell::Zsh,
                "powershell" => Shell::PowerShell,
                "elvish" => Shell::Elvish,
                _ => {
                    eprintln!("Invalid shell");
                    return Err(ExitCode::FAILURE);
                }
            };

            generate(shell, &mut matches, "rrm", &mut std::io::stdout());
            Ok(())
        }

        args::Commands::Set { command } => match command {
            Options::UsePager { value } => {
                match value {
                    true => installer.enable_pager(),
                    false => installer.disable_pager(),
                };
                Ok(())
            }

            Options::GamePath { value } => {
                let gp = match GamePath::try_from(value) {
                    Ok(gp) => gp,
                    Err(_e) => return Err(ExitCode::FAILURE),
                };
                installer.with_rimworld_path(gp);
                Ok(())
            }

            Options::Pager { value } => {
                installer.with_pager(format!("{}", value.display()));
                Ok(())
            }
        },

        args::Commands::Pull { args, ignored } => {
            pull::pull(args, installer, ignored).await;
            Ok(())
        }

        args::Commands::List { display } => {
            list::list(installer, display);
            Ok(())
        }

        args::Commands::Search { command } => match command {
            args::Search::Local { args } => {
                search::search_locally(installer, args);
                Ok(())
            }
            args::Search::Steam { args } => {
                search::search_steam(installer, args).await;
                Ok(())
            }
        },

        args::Commands::SearchLocally { args } => {
            search::search_locally(installer, args);
            Ok(())
        }

        args::Commands::SearchSteam { args } => {
            search::search_steam(installer, args).await;
            Ok(())
        }

        args::Commands::Install { args } => {
            install::install(args, installer, 0, HashSet::new()).await;
            Ok(())
        }
    }
}
