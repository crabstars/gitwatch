use crate::commands::Commands;
use crate::config::{Config, OperationsFacade};

pub trait Displayer {
    fn repositories(&self);
    fn repository_files(&self, repo_name: &str);
    fn info(&self);
}

impl Displayer for Config {
    fn repositories(&self) {
        if let Some(repos) = &self.repositories {
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

    fn repository_files(&self, repo_name: &str) {
        let repo = self.get_repo_not_mut(repo_name);
        println!("Repository path: {}", repo.path);
        if let Some(files) = &repo.files {
            for i in 0..files.len() {
                println!("{}. relativ path: {}", i + 1, files.get(i).unwrap(),)
            }
        } else {
            println!("No files added yet.")
        }
    }

    fn info(&self) {
        let repo_count = match self.repositories.as_ref() {
            Some(repos) => repos.len(),
            None => 0,
        };

        println!(
            "Setting information:\n config path: {}\n logging path: {}\n repository count: {}",
            self.get_dir().display(),
            self.logging_path
                .clone()
                .unwrap_or_else(|| "Logging not set!".to_string()),
            repo_count
        )
    }
}
