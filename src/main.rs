// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
extern crate clap;
extern crate directories;
extern crate num_cpus;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

pub mod config;
pub mod operation;

use std::io::prelude::*;
use std::path::PathBuf;

use clap::{App, Arg};
use config::load_config;
use directories::ProjectDirs;
use operation::copy_to_target;

fn main() {
    // Reads the command-line arguments using clap
    let options = App::new("backup-rat")
        .version("0.3.1")
        .author("System.rat <system.rodent@gmail.com>")
        .about("A versatile backup program")
        .arg(
            Arg::with_name("TARGET")
                .help("The target to backup (the 'all' target backs-up all the auto targets)")
                .required(true)
                .index(1),
        )
        .get_matches();
    println!(
        r"
    /¯¯\          /¯¯\
    \_\ \_------_/ /_/
      \_ ___  ___ _/
  /|____\|_|  |_|/____|\
 /       \      /       \
 \  ______\    /______  /
  \|      =\  /=      |/
            ¯¯
     BACKING UP DATA.
     PLEASE STAND-BY.
     "
    );

    let mut has_targets = false;
    let config = load_config(get_config_folder().join("config.toml"));

    // The all target has been invoked
    if options.value_of("TARGET").unwrap() == "all" {
        for target in config.targets {
            if target.optional {
                continue;
            };
            has_targets = true;
            print!("Backing up target: ");
            if let Some(tag) = &target.tag {
                print!("{}... ", tag);
            } else {
                print!("{}... ", &target.path.display());
            }
            flush();
            let mut threads = 1;
            if config.multi_threaded {
                threads = config.threads;
            }
            // Per-target override
            if let Some(threaded) = target.multi_threaded {
                if threaded {
                    threads = config.threads;
                } else {
                    threads = 1;
                }
            }
            let res = copy_to_target(&target, threads);
            if let Ok(num) = res {
                println!("Done: {} files copied.", num);
            } else {
                println!(" Error: {}", res.unwrap_err());
            }
        }
    // Another target has been invoked
    } else {
        for target in config.targets {
            if let Some(tag) = &target.tag {
                if tag == options.value_of("TARGET").unwrap() {
                    has_targets = true;
                    print!("Backing up target: {}... ", tag);
                    flush();
                    let mut threads = 1;
                    if config.multi_threaded {
                        threads = config.threads;
                    }
                    // Per-target override
                    if let Some(threaded) = target.multi_threaded {
                        if threaded {
                            threads = config.threads;
                        } else {
                            threads = 1;
                        }
                    }
                    let res = copy_to_target(&target, threads);
                    if let Ok(num) = res {
                        println!("Done: {} files copied.", num);
                    } else {
                        println!(" Error: {}", res.unwrap_err());
                    }
                }
            }
        }
    }
    if has_targets {
        println!("\nDone.");
    } else {
        println!("No targets!");
    }
}

fn flush() {
    std::io::stdout().flush().unwrap();
}

fn get_config_folder() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("com", "System.rat", "backup-rat") {
        return PathBuf::from(project_dirs.config_dir());
    } else {
        return PathBuf::new();
    }
}
