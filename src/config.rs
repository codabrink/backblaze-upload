use serde::{Deserialize, Serialize};
use std::{env::var, io::Error, path::PathBuf};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub upload_dir: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub endpoint: String,
    pub bucket: String,
    pub base_url: String,
    pub delete_on_upload: bool,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        let default = Self {
            upload_dir: format!("{}/.up", home()),
            access_key: "<access_key>".to_string(),
            secret_key: "<secret_key>".to_string(),
            region: "us-west-001".to_string(),
            endpoint: "s3.us-west-001.backblazeb2.com".to_string(),
            bucket: "bucket-name".to_string(),
            base_url: "https://s3.us-west-001.backblazeb2.com".to_string(),
            delete_on_upload: true,
        };
        let _ = default.save();
        default
    }
}

fn home() -> String {
    var("HOME").expect("Please set your HOME env var to your home dir.")
}

impl Config {
    pub fn cfg_path() -> PathBuf {
        PathBuf::from(home()).join(".upcfg")
    }
    pub fn load() -> Result<Self, Error> {
        let cfg_path = Self::cfg_path();
        Ok(match cfg_path.exists() {
            true => toml::from_str(&std::fs::read_to_string(cfg_path)?)?,
            false => Config::default(),
        })
    }
    pub fn save(&self) -> Result<(), Error> {
        std::fs::write(Self::cfg_path(), toml::to_string_pretty(self).unwrap())
    }
}
