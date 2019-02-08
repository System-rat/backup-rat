// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
mod cli;
mod config;
mod operations;

use std::io::prelude::*;

use crate::config::{get_config_folder, load_config};

fn main() {
    // Reads the command-line arguments using clap
    let options = cli::get_cli().get_matches();

    let config: config::Config = load_config(get_config_folder().join("config.toml"));

    if let Some(options) = options.subcommand_matches("backup") {
        let mut has_targets = false;
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
        if let Some(target_str) = options.value_of("TARGET") {
            // A target has been invoked
            for target in config.targets {
                if let Some(tag) = &target.tag {
                    if tag == target_str {
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
        } else {
            // The all target has been invoked
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
        }
        if has_targets {
            println!("\nDone.");
        } else {
            println!("No targets!");
        }
    } else if let Some(options) = options.subcommand_matches("completion") {
        cli::print_completions(options.value_of("SHELL").unwrap().to_owned());
    } else if let Some(options) = options.subcommand_matches("restore") {
        let target_str = options.value_of("TARGET").unwrap();
        for target in config.targets {
            if let Some(tag) = &target.tag {
                if tag == target_str {
                    print!("Restoring target: {}... ", tag);
                    flush();
                    let res = target.restore();
                    if let Ok(num) = res {
                        println!("Done: {} files copied.", num);
                    } else {
                        println!(" Error: {}", res.unwrap_err());
                    }
                }
            }
        }
    }
}

fn flush() {
    std::io::stdout().flush().unwrap();
}
