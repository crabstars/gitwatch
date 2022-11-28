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
                    commands::remove_file_from_repo(&sc.name, &sc.file.unwrap(), &mut config.load())
                } else {
                    println!("No relativ file path was specified. Please add to the command a file with \"-f <relativ_file_path_from_repo>\"")
                }
            }
            clap_models::Type::Repo => commands::remove_repo(&sc.name, &mut config.load()),
        },
        clap_models::SubCommand::Init(sc) => {
            commands::init_repo(sc.name, sc.path, sc.branch, &mut config.load())
        }
        clap_models::SubCommand::Set(sc) => match sc.property {
            clap_models::Property::Push => commands::change_auto_push(&sc.name, &mut config.load()),
            clap_models::Property::Active => commands::change_active(&sc.name, &mut config.load()),
            clap_models::Property::Branch => {
                commands::set_branch(&sc.name, sc.branch, &mut config.load())
            }
        },
        clap_models::SubCommand::Add(sc) => {
            commands::add_file_to_repo(sc.name, sc.file, &mut config.load())
        }
        clap_models::SubCommand::List(sc) => match sc.r#type {
            clap_models::TypePlural::Files => {
                if sc.name.is_some() {
                    display::repository_files(&sc.name.unwrap(), &config.load())
                } else {
                    println!("No repository name was specified. Please add to the command a name with \"-n <repo_name>\"")
                }
            }
            clap_models::TypePlural::Repos => display::repositories(&config.load()),
        },
        clap_models::SubCommand::Watch(_) => {
            let config = &config.load();
            if config.logging_path.is_some() {
                logger::init(config.logging_path.clone().unwrap());
            }
            commands::run_gitwatch(config)
        }
        clap_models::SubCommand::Info(_) => display::info(config.load()),
    }
}
