// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Loads the config file into a config struct
//! for easy reading

use std::default::Default;
use std::env::home_dir;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Deserialize)]
struct InnerConfig {
    pub multi_threaded: Option<bool>,
    pub threads: Option<i32>,
    pub target: Option<Vec<InnerBackupTarget>>,
    pub daemon_interval: Option<i32>,
    pub color: Option<bool>,
    pub fancy_text: Option<bool>,
    pub verbose: Option<bool>,
    pub runtime_folder: Option<PathBuf>,
}

#[derive(Deserialize)]
struct InnerBackupTarget {
    pub tag: Option<String>,
    pub multi_threaded: Option<bool>,
    pub path: PathBuf,
    pub ignore_files: Option<Vec<String>>,
    pub ignore_folders: Option<Vec<String>>,
    pub target_path: PathBuf,
    pub optional: Option<bool>,
    pub keep_num: Option<i32>,
    pub always_copy: Option<bool>,
}

/// The config object that contains the relevant configuration
#[derive(Debug, PartialEq)]
pub struct Config {
    pub multi_threaded: bool,
    pub threads: i32,
    pub targets: Vec<BackupTarget>,
    pub daemon_interval: i32,
    pub color: bool,
    pub fancy_text: bool,
    pub verbose: bool,
    pub runtime_folder: PathBuf,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            multi_threaded: true,
            threads: ::num_cpus::get() as i32,
            targets: Vec::new(),
            daemon_interval: 0,
            color: false,
            fancy_text: true,
            verbose: false,
            runtime_folder: home_dir().expect("Can't find home dir").join(".backup_rat"),
        }
    }
}

/// The config sub-struct that contains information about a
/// backup target
#[derive(Debug, PartialEq)]
pub struct BackupTarget {
    pub tag: Option<String>,
    pub multi_threaded: Option<bool>,
    pub path: PathBuf,
    pub ignore_files: Vec<String>,
    pub ignore_folders: Vec<String>,
    pub target_path: PathBuf,
    pub optional: bool,
    pub keep_num: i32,
    pub always_copy: bool,
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
        let raw_config: Result<InnerConfig, _> = ::toml::from_str(&file);
        if let Ok(raw_config) = raw_config {
            let mut targets: Vec<BackupTarget> = Vec::new();
            if let Some(raw_targets) = raw_config.target {
                for target in raw_targets {
                    targets.push(BackupTarget {
                        tag: target.tag,
                        multi_threaded: target.multi_threaded,
                        path: target.path,
                        ignore_files: target.ignore_files.unwrap_or(Vec::new()),
                        ignore_folders: target.ignore_folders.unwrap_or(Vec::new()),
                        target_path: target.target_path,
                        optional: target.optional.unwrap_or(false),
                        keep_num: target.keep_num.unwrap_or(1),
                        always_copy: target.always_copy.unwrap_or(false),
                    });
                }
            }
            Config {
                multi_threaded: raw_config.multi_threaded.unwrap_or(true),
                threads: raw_config.threads.unwrap_or(::num_cpus::get() as i32),
                targets: targets,
                daemon_interval: raw_config.daemon_interval.unwrap_or(0),
                color: raw_config.color.unwrap_or(false),
                fancy_text: raw_config.fancy_text.unwrap_or(true),
                verbose: raw_config.verbose.unwrap_or(false),
                runtime_folder: raw_config
                    .runtime_folder
                    .unwrap_or(home_dir().expect("Can't find home dir").join(".backup_rat")),
            }
        } else {
            Config::default()
        }
    } else {
        Config::default()
    };
    conf
}
