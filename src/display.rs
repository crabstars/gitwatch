use crate::commands::get_repo_not_mut;
use crate::config::{get_dir, Config};

pub fn repositories(config: &Config) {
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

pub fn repository_files(repo_name: &str, config: &Config) {
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

pub fn info(config: Config) {
    println!(
        "Setting information:\n config path: {}\n logging path: {}\n repository count: {}",
        get_dir().display(),
        config
            .logging_path
            .unwrap_or_else(|| "Logging not set!".to_string()),
        config.repositories.unwrap_or_default().len()
    )
}
