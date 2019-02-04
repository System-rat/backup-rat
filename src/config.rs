// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Loads the config file into a config struct
//! for easy reading

use directories::{BaseDirs, ProjectDirs};
use serde_derive::Deserialize;
use std::default::Default;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    #[serde(default = "Vec::new")]
    #[serde(rename = "target")]
    pub targets: Vec<BackupTarget>,
    #[serde(default = "default_daemon_interval")]
    pub daemon_interval: i32,
    #[serde(default = "default_color")]
    pub color: bool,
    #[serde(default = "default_fancy_text")]
    pub fancy_text: bool,
    #[serde(default = "default_verbose")]
    pub verbose: bool,
    #[serde(default = "default_runtime_folder")]
    pub runtime_folder: PathBuf,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct BackupTarget {
    pub tag: Option<String>,
    pub path: PathBuf,
    #[serde(default = "Vec::new")]
    pub ignore_files: Vec<String>,
    #[serde(default = "Vec::new")]
    pub ignore_folders: Vec<String>,
    pub target_path: PathBuf,
    #[serde(default = "default_optional")]
    pub optional: bool,
    #[serde(default = "default_keep_num")]
    pub keep_num: i32,
    #[serde(default = "default_always_copy")]
    pub always_copy: bool,
    #[serde(flatten)]
    pub additional_options: Option<Additional>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum Additional {
    Network { url: String, password: String },
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SharedOptions {}

impl BackupTarget {
    pub fn backup(self) -> std::io::Result<i32> {
        if let Some(ado) = self.additional_options {
            match ado {
                Additional::Network { .. } => unimplemented!(),
            }
        } else {
            crate::operations::local_copy(self)
        }
    }
}

// ***************
// Config defaults
// ***************
const fn default_daemon_interval() -> i32 {
    0
}

const fn default_color() -> bool {
    true
}

const fn default_fancy_text() -> bool {
    true
}

const fn default_verbose() -> bool {
    false
}

fn default_runtime_folder() -> PathBuf {
    BaseDirs::new()
        .expect("Could not get base directories")
        .home_dir()
        .join(".backup_rat")
}

// ***************
// Target defaults
// ***************
const fn default_optional() -> bool {
    false
}

const fn default_keep_num() -> i32 {
    1
}

const fn default_always_copy() -> bool {
    false
}

impl Default for Config {
    fn default() -> Config {
        Config {
            daemon_interval: default_daemon_interval(),
            color: default_color(),
            fancy_text: default_fancy_text(),
            verbose: default_verbose(),
            runtime_folder: default_runtime_folder(),
            targets: Vec::new(),
        }
    }
}

/// Loads the config file and returns the Config struct
///
/// # Parameters
/// - config_path: the PathBuf (or just path) to the
/// target config file
///
/// # Note
/// if the config is invalid the default
/// configuration is returned
pub fn load_config(config_path: PathBuf) -> Config {
    let file = read_to_string(config_path);
    let conf: Config = if let Ok(file) = file {
        toml::from_str(file.as_ref()).unwrap_or_default()
    } else {
        Config::default()
    };
    conf
}

pub fn get_config_folder() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("com", "System.rat", "backup-rat") {
        return PathBuf::from(project_dirs.config_dir());
    } else {
        return PathBuf::new();
    }
}

#[test]
fn loading_from_string() {
    let target_config = Config {
        color: false,
        fancy_text: false,
        daemon_interval: 100,
        targets: vec![BackupTarget {
            tag: Some("target1".to_owned()),
            path: PathBuf::from("/etc"),
            target_path: PathBuf::from("/mnt/backup"),
            keep_num: 1,
            always_copy: false,
            optional: false,
            ignore_files: Vec::new(),
            ignore_folders: Vec::new(),
            additional_options: Some(Additional::Network {
                // Not an actual url mind you
                url: "www.test.com".to_owned(),
                password: "test".to_owned(),
            }),
        }],
        verbose: true,
        runtime_folder: PathBuf::from("/"),
    };
    let generated_config = toml::from_str(
        "
        color = false
        fancy_text = false
        daemon_interval = 100
        verbose = true
        runtime_folder = \"/\"

        [[target]]
        tag = \"target1\"
        path = \"/etc\"
        target_path = \"/mnt/backup\"
        url = \"www.test.com\"
        password = \"test\"
    ",
    )
    .unwrap();

    assert_eq!(target_config, generated_config);
}
