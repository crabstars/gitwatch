use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use std::{fs, str};

#[derive(Serialize, Deserialize, Debug)]
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

pub fn load() -> Config {
    let file_content = match fs::read_to_string(get_dir()) {
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

pub fn save(config: &Config) {
    let serialized = serde_yaml::to_string(&config).unwrap();
    fs::write(get_dir(), serialized).expect("Unable to write file");
}

pub fn create_default(overwrite: bool) {
    let config_path = get_dir();

    if config_path.exists() && !overwrite {
        panic!(
            "File already exists. Please call command with --overwrite if you want to replace it"
        )
    }

    File::create(&config_path).unwrap();
    let config = Config {
        repositories: None,
        logging_path: Some(String::from("/tmp/gitwatch-log/output.log")),
    };
    save(&config);
    println!("File was created: {:?}", config_path)
}

pub fn get_dir() -> PathBuf {
    let mut home_dir = match home::home_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home dir!"),
    };
    home_dir.push(".config/gitwatch-rs/config.yaml");
    home_dir
}
