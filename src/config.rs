#[cfg(test)]
use mockall::{automock, predicate::*};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use std::{fs, str};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub repositories: Option<Vec<GitRepository>>,
    pub logging_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitRepository {
    pub name: String,
    pub branch: String,
    pub path: String,
    pub files: Option<Vec<String>>,
    pub auto_push: bool,
    pub active: bool,
}

pub trait OperationsFacade {
    fn save(&self);
    fn create_default(&mut self, overwrite: bool);
    fn load(&self) -> Self;
    fn get_dir(&self) -> PathBuf;
}

trait Operations {
    fn save(&self);
    fn create_default(&mut self, overwrite: bool);
    fn load(&self) -> Self;
    fn get_dir(&self, home_lib: &impl HomeLib) -> PathBuf;
}

impl OperationsFacade for Config {
    fn save(&self) {
        self::Operations::save(self);
    }

    fn create_default(&mut self, overwrite: bool) {
        self::Operations::create_default(self, overwrite);
    }

    fn load(&self) -> Self {
        self::Operations::load(self)
    }

    fn get_dir(&self) -> PathBuf {
        self::Operations::get_dir(self, &HomeFacade {})
    }
}

impl Operations for Config {
    fn save(&self) {
        let serialized = serde_yaml::to_string(&self).unwrap();
        fs::write(self::OperationsFacade::get_dir(self), serialized).expect("Unable to write file");
    }

    fn create_default(&mut self, overwrite: bool) {
        let config_path = self::OperationsFacade::get_dir(self);

        if config_path.exists() && !overwrite {
            panic!(
                "File already exists. Please call command with --overwrite if you want to replace it"
            )
        }

        File::create(&config_path).unwrap();
        self.repositories = None;
        self.logging_path = Some(String::from("/tmp/gitwatch-log/output.log"));

        self::Operations::save(self);
        println!("File was created: {:?}", config_path)
    }

    fn load(&self) -> Self {
        let file_content = match fs::read_to_string(self::OperationsFacade::get_dir(self)) {
            Ok(content) => content,
            Err(_) => {
                let msg =  "Failed to read config file. Please run 'gitwatch init-config' or create the following path: $USER/.config/gitwatch-rs/gitwatch.yaml and add the values from https://github.com/crabstars/gitwatch/config-example.yaml";
                panic!("{}", msg);
            }
        };

        match serde_yaml::from_str(&file_content) {
            Ok(config) => config,
            Err(e) => panic!("Could not read config: {}", e),
        }
    }

    fn get_dir(&self, home_lib: &impl HomeLib) -> PathBuf {
        let mut home_dir = match home_lib.home_dir() {
            Some(path) => path,
            None => panic!("Impossible to get your home dir!"),
        };
        home_dir.push(".config/gitwatch-rs/config.yaml");
        home_dir
    }
}

#[cfg_attr(test, automock)]
trait HomeLib {
    fn home_dir(&self) -> Option<PathBuf>;
}
struct HomeFacade {}
impl HomeLib for HomeFacade {
    fn home_dir(&self) -> Option<PathBuf> {
        home::home_dir()
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::path;

    use crate::config::{Config, MockHomeLib, Operations};
    use std::path::PathBuf;

    #[test]
    fn get_dir_returns_correct_dir_mock() {
        let home_test = "/home/test/";
        let mut mock = MockHomeLib::new();

        let mut path = PathBuf::new();
        path.push(home_test);
        mock.expect_home_dir().once().return_const(path);

        let config = Config::default();

        assert_eq!(
            config.get_dir(&mock).to_str().unwrap(),
            home_test.to_owned() + ".config/gitwatch-rs/config.yaml"
        );
    }
}
