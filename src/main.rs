use crate::commands::Commands;
use crate::display::Displayer;
use clap::Parser;
use config::{Config, OperationsFacade};
mod clap_models;
mod commands;
mod config;
mod display;
mod logger;

/*
commands:

gitwatch set-intervall-cronie ***** (crontab syntax) -> can only be used if cronie is installed
gitwatch set-intervall-crontab ***** (crontab syntax) -> can only be used if crontab is installed

installer script:
    call create_default_config()

test:
    todo

Future:
    - delete older logs
    - change commit message, default value from config maybe
*/

fn main() {
    let command = clap_models::Command::parse();
    let mut config = Config::default();

    match command.subcmd {
        clap_models::SubCommand::New(sc) => config.create_default(sc.overwrite),
        clap_models::SubCommand::Rm(sc) => match sc.r#type {
            clap_models::Type::File => {
                if sc.file.is_some() {
                    config.load();
                    config.remove_file_from_repo(&sc.name, &sc.file.unwrap())
                } else {
                    println!("No relativ file path was specified. Please add to the command a file with \"-f <relativ_file_path_from_repo>\"")
                }
            }
            clap_models::Type::Repo => {
                config.load();
                config.remove_repo(&sc.name)
            }
        },
        clap_models::SubCommand::Init(sc) => {
            config.load();
            config.init_repo(sc.name, sc.path, sc.branch)
        }

        clap_models::SubCommand::Set(sc) => match sc.property {
            clap_models::Property::Push => {
                config.load();
                config.change_auto_push(&sc.name)
            }

            clap_models::Property::Active => {
                config.load();
                config.change_active(&sc.name)
            }

            clap_models::Property::Branch => {
                config.load();
                config.set_branch(&sc.name, sc.branch)
            }
        },
        clap_models::SubCommand::Add(sc) => {
            config.load();
            config.add_file_to_repo(sc.name, sc.file)
        }
        clap_models::SubCommand::List(sc) => match sc.r#type {
            clap_models::TypePlural::Files => {
                if sc.name.is_some() {
                    config.load();
                    config.repository_files(&sc.name.unwrap())
                } else {
                    println!("No repository name was specified. Please add to the command a name with \"-n <repo_name>\"")
                }
            }
            clap_models::TypePlural::Repos => {
                config.load();
                config.repositories()
            }
        },
        clap_models::SubCommand::Watch(_) => {
            config.load();
            if config.logging_path.is_some() {
                logger::init(config.logging_path.clone().unwrap());
            }
            config.run_gitwatch();
        }
        clap_models::SubCommand::Info(_) => {
            config.load();
            config.info()
        }
    }
}
