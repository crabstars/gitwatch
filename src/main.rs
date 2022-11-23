use core::panic;
use git2::{Repository, Status};
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, vec};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    repositories: Option<Vec<GitRepository>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitRepository {
    name: String,
    path: String,
    files: Option<Vec<String>>,
    auto_push: bool,
    active: bool,
}

/*
commands:

gitwatch config-init  => creates the config file (done)
gitwatch init [name] [path] adds repo to gitwatch tracker,  name has to be unique (done)
gitwatch set push true/false  (done)
gitwatch add [name] [path-to-file] should be relativ from main repo (done)
gitwatch rm [name] [path-to-file] should be relativ from main repo, removes file from tracking (done)
gitwatch delete [name] removes repo from tracking and all files (done)
gitwatch list --> returns all repo names and path
gitwatch list-files [repo-name] --> all files which are currently being tracked


gitwatch set-intervall-cronie ***** (crontab syntax) -> can only be used if cronie is installed
gitwatch set-intervall-crontab ***** (crontab syntax) -> can only be used if crontab is installed


installer:
    call create_default_config()

test:
    todo
*/

fn main() {
    //create_default_config(false);
    let mut config = load_config();

    //remove_file_from_repo("gitwatch", "kekl.new", &mut config);
    //remove_repo("gitwatch", &mut config)
    print_repos(&config);
    /*
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


    match &r.files {
                Some(mut files) => files.push(relativ_file_path.clone()),
                None => r.files = Some(vec![relativ_file_path.clone()]),
            }
     */
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
            auto_push: true,
            active: true,
        })
    } else {
        config.repositories = Some(vec![GitRepository {
            name,
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
    for repo in config.repositories.iter_mut().flatten() {
        if repo.name == repo_name {
            if let Some(ref mut files) = repo.files {
                files.push(relativ_file_path.clone())
            } else {
                repo.files = Some(vec![relativ_file_path.clone()])
            }
        }
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

fn get_repo<'a>(repo_name: &str, config: &mut Config) -> &'a mut GitRepository {
    for repo in config.repositories.iter_mut().flatten() {
        if repo.name != repo_name {
            return repo;
        }
    }
    panic!("No repo found with given name.")
}

fn change_auto_push(repo_name: &str, config: &mut Config) {
    let repo = get_repo(repo_name, config);
    repo.auto_push = !repo.auto_push;
    //config.auto_push = auto_push;
    // TODO add repo name// flip auto_push and print new bool
    save_config(config)
}

fn update_activation(repo_name: &str, config: &mut Config) {
    let repo = get_repo(repo_name, config);
    repo.active = !repo.active;
    //config.auto_push = auto_push;
    // TODO add repo name// flip activation and print new bool
    save_config(config)
}

fn print_repos(config: &Config) {
    if let Some(repos) = &config.repositories {
        for i in 0..repos.len() {
            println!(
                "{}. name: {}; path: {}; auto_push: {}; active: {}",
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

fn print_files_from_repo(config: &Config) {}
