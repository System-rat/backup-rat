// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
mod config;
mod operations;

use std::io::prelude::*;

use crate::config::{get_config_folder, load_config};
use clap::{App, Arg};

fn main() {
    // Reads the command-line arguments using clap
    let options = App::new("backup-rat")
        .version("0.5.0")
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
    let config: config::Config = load_config(get_config_folder().join("config.toml"));

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
            let res = target.backup();
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
                    let res = target.backup();
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
