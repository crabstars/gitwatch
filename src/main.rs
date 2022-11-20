use core::panic;
use git2::{Repository, Status};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    repositories: Option<Vec<GitRepository>>,
    auto_push: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitRepository {
    name: String,
    path: String,
    files: Option<Vec<String>>,
}

/*
commands:

gitwatch config-init  => creates the config file
gitwatch init [name] [path] adds repo to gitwatch tracker,  name has to be unique (done)
gitwatch set push true/false
gitwatch add [name] [path-to-file] should be relativ from main repo
gitwatch rm [name] [path-to-file] should be relativ from main repo, removes fiel from tracking
gitwatch delete [name] removes repo from tracking and all files

gitwatch set-intervall-cronie ***** (crontab syntax) -> can only be used if cronie is installed
gitwatch set-intervall-crontab ***** (crontab syntax) -> can only be used if crontab is installed

*/

fn main() {
    let mut config = load_config();

    let repo = match Repository::open("/home/kami/Documents/Coding/Rust/gitwatch") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };
    let path = Path::new("test.txt");
    match repo.status_file(path) {
        Ok(status) => println!("{:?}", status == Status::WT_MODIFIED),
        Err(e) => panic!("failed to get file status: {}", e),
    };

    init_repo(
        String::from("new test2"),
        String::from("/home/kami/Documents/Coding/Rust/gitwatch"),
        &mut config,
    );
}

fn get_config_dir() -> PathBuf {
    let mut home_dir = match home::home_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home dir!"),
    };
    home_dir.push(".config/gitwatch-rs/gitwatch.yaml");
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

fn init_repo(name: String, path: String, config: &mut Config) {
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
            path,
            files: None,
        })
    } else {
        config.repositories = Some(vec![GitRepository {
            name,
            path,
            files: None,
        }])
    }
    save_config(config)
}
