// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
extern crate backup_rat;
extern crate clap;

use std::io::prelude::*;
use std::path::PathBuf;

use backup_rat::config::load_config;
use backup_rat::operation::copy_to_target;
use clap::{App, Arg};

fn main() {
    // Reads the command-line arguments using clap
    let options = App::new("backup-rat")
        .version("0.1.2")
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

    let mut did_backup = false;
    let config = load_config(get_config_folder().join("config.toml"));

    // The all target has been invoked
    if options.value_of("TARGET").unwrap() == "all" {
        for target in config.targets {
            if target.optional {
                continue;
            };
            print!("Backing up target: ");
            if let Some(tag) = &target.tag {
                print!("{}", tag);
            } else {
                print!("{}", &target.path.display());
            }
            flush();
            let res = copy_to_target(&target);
            if res.is_err() {
                println!(" Error: {}", res.unwrap_err());
            } else {
                println!("");
                did_backup = true;
            }
        }
    // Another target has been invoked
    } else {
        for target in config.targets {
            if let Some(tag) = &target.tag {
                if tag == options.value_of("TARGET").unwrap() {
                    print!("Backing up target: {}", tag);
                    flush();
                    let res = copy_to_target(&target);
                    if res.is_err() {
                        println!(" Error: {}", res.unwrap_err());
                    } else {
                        println!("");
                        did_backup = true;
                    }
                }
            }
        }
    }
    if did_backup {
        println!("\nDone.");
    } else {
        println!("No targets!");
    }
}

fn flush() {
    std::io::stdout().flush().unwrap();
}

// if only windows followed SOME *NIX standards
#[cfg(target_os = "windows")]
fn get_config_folder() -> PathBuf {
    ::std::env::home_dir().unwrap().join("AppData/backup-rat")
}

#[cfg(not(target_os = "windows"))]
fn get_config_folder() -> PathBuf {
    ::std::env::home_dir().unwrap().join(".config/backup-rat")
}
