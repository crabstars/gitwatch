use clap::{ArgAction, Parser, Subcommand};
use core::panic;
use git2::{Repository, Status};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command as TerminalCommand;
use std::{fs, str, vec};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    repositories: Option<Vec<GitRepository>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitRepository {
    name: String,
    branche: String,
    path: String,
    files: Option<Vec<String>>,
    auto_push: bool,
    active: bool,
}

/*
commands:

gitwatch info => config path and how many repos

gitwatch set-intervall-cronie ***** (crontab syntax) -> can only be used if cronie is installed
gitwatch set-intervall-crontab ***** (crontab syntax) -> can only be used if crontab is installed
gitwatch change branche for commits

installer:
    call create_default_config()

test:
    todo

Future:
    - change commit message, default value from config maybe
    - add branche for repo where the commit should happen
*/
#[derive(Parser)]
#[clap(version = "1.0")]
struct Command {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    New(New),
    Rm(Remove),
    Init(Init),
    Set(Set),
    Add(Add),
    List(List),
    Watch(Watch),
}

#[derive(Parser)]
pub struct Watch {}

#[derive(Parser)]
pub struct New {
    // Overwrite existing config
    #[arg(short, long, action= ArgAction::Set, default_value_t = false)]
    overwrite: bool,
}

#[derive(Parser)]
pub struct Init {
    // unique name for repo
    #[clap(short, long)]
    name: String,

    // unique name for repo
    #[clap(short, long)]
    branche: String,

    // absolute path for repo location
    #[clap(short, long)]
    path: String,
}

#[derive(Parser)]
pub struct Remove {
    // file or repo
    #[clap(short, long, value_enum)]
    r#type: Type,

    // unique name for repo
    #[clap(short, long)]
    name: String,

    // relativ file path from repo
    #[clap(short, long)]
    file: Option<String>,
}

#[derive(Parser)]
pub struct Add {
    // unique name for repo
    #[clap(short, long)]
    name: String,

    // relativ file path from repo
    #[clap(short, long)]
    file: String,
}

#[derive(Parser)]
pub struct List {
    // file or repo
    #[clap(short, long, value_enum)]
    r#type: TypePlural,

    // unique name for repo
    #[clap(short, long)]
    name: Option<String>,
}

#[derive(Parser)]
pub struct Set {
    // unique name for repo
    #[clap(short, long)]
    name: String,

    // remove file or repo
    #[clap(short, long, value_enum)]
    property: Property,
}

#[derive(clap::ValueEnum, Clone)]
enum Property {
    Push,
    Active,
}

#[derive(clap::ValueEnum, Clone)]
enum Type {
    File,
    Repo,
}

#[derive(clap::ValueEnum, Clone)]
enum TypePlural {
    Files,
    Repos,
}

fn main() {
    let command = Command::parse();

    match command.subcmd {
        SubCommand::New(sc) => create_default_config(sc.overwrite),
        SubCommand::Rm(sc) => match sc.r#type {
            Type::File => {
                if sc.file.is_some() {
                    remove_file_from_repo(&sc.name, &sc.file.unwrap(), &mut load_config())
                } else {
                    println!("No relativ file path was given. Please add to the command a file with \"-f <relativ_file_path_from_repo>\"")
                }
            }
            Type::Repo => remove_repo(&sc.name, &mut load_config()),
        },
        SubCommand::Init(sc) => init_repo(sc.name, sc.branche, sc.path, &mut load_config()),
        SubCommand::Set(sc) => match sc.property {
            Property::Push => change_auto_push(&sc.name, &mut load_config()),
            Property::Active => change_active(&sc.name, &mut load_config()),
        },
        SubCommand::Add(sc) => add_file_to_repo(sc.name, sc.file, &mut load_config()),
        SubCommand::List(sc) => match sc.r#type {
            TypePlural::Files => {
                if sc.name.is_some() {
                    print_files_from_repo(&sc.name.unwrap(), &load_config())
                } else {
                    println!("No repository name was given. Please add to the command a name with \"-n <repo_name>\"")
                }
            }
            TypePlural::Repos => print_repos(&load_config()),
        },
        SubCommand::Watch(_) => run_gitwatch(&load_config()),
    }
}

fn run_gitwatch(config: &Config) {
    if config.repositories.is_none() {
        return;
    }
    // Test File is not added (when commit)
    // file does not exists
    // no rights to push

    for repo in config.repositories.iter().flatten() {
        if !repo.active {
            return;
        }
        let git_repo = match Repository::open(repo.path.clone()) {
            Ok(git_repo) => git_repo,
            Err(e) => {
                println!("failed to open: {}", e);
                continue;
            }
        };

        //  change branche if necessary
        let msg = TerminalCommand::new("git")
            .args(["-C", &repo.path, "checkout", &repo.branche])
            .output()
            .expect("failed to execute process");

        println!("{:?}", str::from_utf8(&msg.stderr).unwrap());

        // commit files
        for file in repo.files.iter().flatten() {
            let path = Path::new(file);
            match git_repo.status_file(path) {
                Ok(status) => {
                    if status == Status::WT_MODIFIED || status == Status::INDEX_NEW {
                        let _ = TerminalCommand::new("git")
                            .args(["-C", &repo.path, "commit", "-m", file, file])
                            .output()
                            .expect("failed to execute process");
                        //println!("{:?}", str::from_utf8(&msg.stdout).unwrap());
                    }
                }
                Err(e) => panic!("failed to get file status: {}", e),
            };
        }

        // push changes
        if !repo.auto_push {
            return;
        }

        let msg = TerminalCommand::new("git")
            .args([
                "-C",
                &repo.path,
                "push",
                "--set-upstream",
                "origin",
                &repo.branche,
            ])
            .output()
            .expect("failed to execute process");

        println!("{:?}", str::from_utf8(&msg.stderr).unwrap());
    }
}

fn get_config_dir() -> PathBuf {
    let mut home_dir = match home::home_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home dir!"),
    };
    home_dir.push(".config/gitwatch-rs/config.yaml");
    home_dir
}

fn load_config() -> Config {
    let file_content = match fs::read_to_string(get_config_dir()) {
        Ok(content) => content,
        Err(_) => panic!("Failed to read config file. Please run 'gitwatch init-config' or create the following path: $USER/.config/gitwatch-rs/gitwatch.yaml and add the values from https://github.com/crabstars/gitwatch/config-example.yaml"),
    };

    match serde_yaml::from_str(&file_content) {
        Ok(config) => config,
        Err(e) => panic!("Could not read config: {}", e),
    }
}

fn save_config(config: &Config) {
    let serialized = serde_yaml::to_string(&config).unwrap();
    fs::write(get_config_dir(), serialized).expect("Unable to write file");
}

fn create_default_config(overwrite: bool) {
    let config_path = get_config_dir();

    if config_path.exists() && !overwrite {
        panic!(
            "File already exists. Please call command with --overwrite if you want to replace it"
        )
    }

    File::create(&config_path).unwrap();
    let config = Config { repositories: None };
    save_config(&config);
    println!("File was created: {:?}", config_path)
}

fn init_repo(name: String, path: String, branche: String, config: &mut Config) {
    match &config.repositories {
        Some(ref repos) => {
            if repos.iter().any(|repo| repo.name == name) {
                panic!("You can't use the same name twice")
            }
        }
        None => (),
    }

    if let Some(ref mut repos) = config.repositories {
        repos.push(GitRepository {
            name,
            branche,
            path,
            files: None,
            auto_push: true,
            active: true,
        })
    } else {
        config.repositories = Some(vec![GitRepository {
            name,
            branche,
            path,
            files: None,
            auto_push: true,
            active: true,
        }])
    }
    save_config(config)
}

fn remove_repo(repo_name: &str, config: &mut Config) {
    config
        .repositories
        .as_mut()
        .expect("There are no repositories to remove!")
        .retain(|repo| repo.name != repo_name);

    save_config(config)
}

fn add_file_to_repo(repo_name: String, relativ_file_path: String, config: &mut Config) {
    // provide repo_name or be in repo directory (TODO)
    // we start only with repo_name

    let repo = get_repo(&repo_name, config);

    if let Some(ref mut files) = repo.files {
        files.push(relativ_file_path)
    } else {
        repo.files = Some(vec![relativ_file_path])
    }

    save_config(config)
}

fn remove_file_from_repo(repo_name: &str, relativ_file_path: &str, config: &mut Config) {
    // provide repo_name or be in repo directory (TODO)
    // we start only with repo_name

    for repo in config.repositories.iter_mut().flatten() {
        if repo.name != repo_name {
            continue;
        }

        repo.files
            .as_mut()
            .expect("There are no files to remove!")
            .retain(|file| file != relativ_file_path);
    }
    save_config(config)
}

fn get_repo<'a>(repo_name: &str, config: &'a mut Config) -> &'a mut GitRepository {
    for repo in config.repositories.iter_mut().flatten() {
        if repo.name == repo_name {
            return repo;
        }
    }
    panic!("No repo found with given name.")
}

fn get_repo_not_mut<'a>(repo_name: &str, config: &'a Config) -> &'a GitRepository {
    for repo in config.repositories.iter().flatten() {
        if repo.name == repo_name {
            return repo;
        }
    }
    panic!("No repo found with given name.")
}

fn change_auto_push(repo_name: &str, config: &mut Config) {
    let repo = get_repo(repo_name, config);
    repo.auto_push = !repo.auto_push;

    println!("Auto push was set to: {}", repo.auto_push);
    save_config(config)
}

fn change_active(repo_name: &str, config: &mut Config) {
    let repo = get_repo(repo_name, config);
    repo.active = !repo.active;

    if repo.active {
        println!("The repo is now active and the programm will commit all new changes.")
    } else {
        println!("The repo is now inactive and no commits or pushes are happening.")
    }
    save_config(config)
}

fn print_repos(config: &Config) {
    if let Some(repos) = &config.repositories {
        for i in 0..repos.len() {
            println!(
                "{}.\n name: {}\n path: {}\n auto_push: {}\n active: {}",
                i + 1,
                repos.get(i).unwrap().name,
                repos.get(i).unwrap().path,
                repos.get(i).unwrap().auto_push,
                repos.get(i).unwrap().active,
            )
        }
    } else {
        println!("No repositories added yet.")
    }
}

fn print_files_from_repo(repo_name: &str, config: &Config) {
    let repo = get_repo_not_mut(repo_name, config);
    println!("Repository path: {}", repo.path);
    if let Some(files) = &repo.files {
        for i in 0..files.len() {
            println!("{}. relativ path: {}", i + 1, files.get(i).unwrap(),)
        }
    } else {
        println!("No files added yet.")
    }
}
