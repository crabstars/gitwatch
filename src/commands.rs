use crate::config::{Config, GitRepository, OperationsFacade};
use core::panic;
use git2::{Repository, Status};
use std::ops::ControlFlow;
use std::path::Path;
use std::process::Command as TerminalCommand;
use std::{str, vec};

pub trait Commands {
    fn run_gitwatch(&self);
    fn init_repo(&mut self, name: String, path: String, branch: String);
    fn remove_repo(&mut self, repo_name: &str);
    fn add_file_to_repo(&mut self, repo_name: String, relativ_file_path: String);
    fn remove_file_from_repo(&mut self, repo_name: &str, relativ_file_path: &str);
    fn change_auto_push(&mut self, repo_name: &str);
    fn change_active(&mut self, repo_name: &str);
    fn set_branch(&mut self, repo_name: &str, branch: Option<String>);
    fn get_repo_not_mut(&self, repo_name: &str) -> &GitRepository;
}

impl Commands for Config {
    fn run_gitwatch(&self) {
        if self.repositories.is_none() {
            return;
        }
        /*
        TODO test:
            - file is not added (when commit)
            - file does not exists
            - no rights to push
        */
        for repo in self.repositories.iter().flatten() {
            if !repo.active {
                return;
            }
            let git_repo = match Repository::open(repo.path.clone()) {
                Ok(git_repo) => git_repo,
                Err(e) => {
                    log::error!("failed to open: {}", e);
                    continue;
                }
            };

            // check if current branch is correct
            if let ControlFlow::Break(_) = check_branch(repo) {
                continue;
            }

            // commit files
            commit(repo, git_repo);

            // push changes
            if repo.auto_push {
                push(repo);
            }
        }
    }

    fn init_repo(&mut self, name: String, path: String, branch: String) {
        if let Some(ref repos) = self.repositories {
            if repos.iter().any(|repo| repo.name == name) {
                panic!("You can't use the same name twice")
            }
        }

        if let Some(ref mut repos) = self.repositories {
            repos.push(GitRepository {
                name,
                branch,
                path,
                files: None,
                auto_push: true,
                active: true,
            })
        } else {
            self.repositories = Some(vec![GitRepository {
                name,
                branch,
                path,
                files: None,
                auto_push: true,
                active: true,
            }])
        }
        self.save()
    }

    fn remove_repo(&mut self, repo_name: &str) {
        self.repositories
            .as_mut()
            .expect("There are no repositories to remove!")
            .retain(|repo| repo.name != repo_name);

        self.save()
    }

    fn add_file_to_repo(&mut self, repo_name: String, relativ_file_path: String) {
        // provide repo_name or be in repo directory (TODO)
        // we start only with repo_name

        let repo = get_repo(&repo_name, self);

        if let Some(ref mut files) = repo.files {
            files.push(relativ_file_path)
        } else {
            repo.files = Some(vec![relativ_file_path])
        }

        self.save()
    }

    fn remove_file_from_repo(&mut self, repo_name: &str, relativ_file_path: &str) {
        // provide repo_name or be in repo directory (TODO)
        // we start only with repo_name

        for repo in self.repositories.iter_mut().flatten() {
            if repo.name != repo_name {
                continue;
            }

            repo.files
                .as_mut()
                .expect("There are no files to remove!")
                .retain(|file| file != relativ_file_path);
        }
        self.save()
    }

    fn change_auto_push(&mut self, repo_name: &str) {
        let repo = get_repo(repo_name, self);
        repo.auto_push = !repo.auto_push;

        println!("Auto push was set to: {}", repo.auto_push);
        self.save()
    }

    fn change_active(&mut self, repo_name: &str) {
        let repo = get_repo(repo_name, self);
        repo.active = !repo.active;

        if repo.active {
            println!("The repo is now active and the programm will commit all new changes.")
        } else {
            println!("The repo is now inactive and no commits or pushes are happening.")
        }
        self.save()
    }

    fn set_branch(&mut self, repo_name: &str, branch: Option<String>) {
        if branch.is_none() {
            println!("No branch given. Specify with \"--branch <name>\" or \"-b <name>\"");
            return;
        }
        let repo = get_repo(repo_name, self);
        repo.branch = branch.unwrap();
        self.save()
    }

    fn get_repo_not_mut(&self, repo_name: &str) -> &GitRepository {
        for repo in self.repositories.iter().flatten() {
            if repo.name == repo_name {
                return repo;
            }
        }
        panic!("No repo found with given name.")
    }
}

fn get_repo<'a>(repo_name: &str, config: &'a mut Config) -> &'a mut GitRepository {
    for repo in config.repositories.iter_mut().flatten() {
        if repo.name == repo_name {
            return repo;
        }
    }
    panic!("No repo found with given name.")
}

fn push(repo: &GitRepository) {
    let msg = TerminalCommand::new("git")
        .args([
            "-C",
            &repo.path,
            "push",
            "--set-upstream",
            "origin",
            &repo.branch,
        ])
        .output()
        .expect("failed to execute process");
    log::info!("{:?}", str::from_utf8(&msg.stderr).unwrap().trim_end());
}

fn commit(repo: &GitRepository, git_repo: Repository) {
    repo.files.iter().flatten().for_each(|file| {
        let path = Path::new(file);
        match git_repo.status_file(path) {
            Ok(status) => {
                if status == Status::WT_MODIFIED || status == Status::INDEX_NEW {
                    let msg = TerminalCommand::new("git")
                        .args(["-C", &repo.path, "commit", "-m", file, file])
                        .output()
                        .expect("failed to execute process");
                    log::info!("{:?}", str::from_utf8(&msg.stdout).unwrap().trim_end());
                }
            }
            Err(e) => log::error!("failed to get file status: {}", e),
        };
    });
}

fn check_branch(repo: &GitRepository) -> ControlFlow<()> {
    let msg = TerminalCommand::new("git")
        .args(["-C", &repo.path, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("failed to execute process");
    let current_branch = str::from_utf8(&msg.stdout).unwrap().trim_end();
    if repo.branch != current_branch {
        log::warn!(
            "Repo: {} skips repo because current branch: \"{}\" is not the defined one: \"{}\".",
            repo.name,
            current_branch,
            repo.branch
        );
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}
