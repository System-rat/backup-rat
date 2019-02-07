// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Contains helper methods and structs for backup operations
//! such as checking timestamps, copying to targets and restoring

use std::fs::{copy, create_dir, read_dir, remove_dir_all};
use std::fs::{DirEntry, File, Metadata};
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use regex::Regex;

/// Checks if a directory or file is ignored
///
/// # Parameters
/// - path: The path (relative to the base directory) of the folder or file
/// in question
/// - ignored_files: The vector of files to be ignored (regexes are prefixed
/// with a r#)
/// - ignored_folders: same as `ignored_files` except for directories
///
/// # Returns
/// if the file or folder is to be ignored
pub fn ignored(
    path: &PathBuf,
    metadata: &Metadata,
    ignored_files: &[String],
    ignored_folders: &[String],
) -> bool {
    if metadata.is_file() {
        for filter in ignored_files {
            if filter.starts_with("r#") {
                let r = Regex::new(&filter[2..]);
                if let Ok(r) = r {
                    if r.is_match(path.file_name().unwrap().to_str().unwrap()) {
                        return true;
                    }
                }
            } else if path.file_name().unwrap().to_str().unwrap() == filter {
                return true;
            }
        }
    } else {
        for filter in ignored_folders {
            if filter.starts_with("r#") {
                let r = Regex::new(&filter[2..]);
                if let Ok(r) = r {
                    if r.is_match(path.as_os_str().to_str().unwrap()) {
                        return true;
                    }
                }
            } else if path.as_os_str().to_str().unwrap() == filter {
                return true;
            }
        }
    }
    false
}

/// Copies a folder or file to a destination
/// whilst also checking timestamps to override or not
///
/// # Parameters
/// - target: the SharedOptions from a Local target
///
/// # Returns
/// Returns an the number of copied files
///
/// # Error
/// Returns an error if the target could not be written to or the *from* target could
/// not be read
pub fn local_copy(
    from: PathBuf,
    to: PathBuf,
    check_timestamp: bool,
    keep_num: i32,
    ignored_files: Vec<String>,
    ignored_folders: Vec<String>,
) -> Result<i32> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let check_timestamp = if keep_num == 1 {
        check_timestamp
    } else {
        false
    };
    let mut num: i32 = 0;
    let from_file: File = File::open(&from)?;
    if File::open(&to).is_err() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "The destination is unavailable.",
        ));
    };

    if from_file.metadata().unwrap().is_file() {
        if let Some(file_name) = from.file_name() {
            if ignored(
                &from,
                &from_file.metadata().unwrap(),
                &ignored_files,
                &ignored_folders,
            ) {
                return Ok(0);
            }
            if let Ok(to_file) = File::open(&to.join(file_name)) {
                if check_timestamp
                    && from_file.metadata().unwrap().modified().unwrap()
                        < to_file.metadata().unwrap().modified().unwrap()
                {
                    return Ok(0);
                }
            }
            num += 1;
            copy(&from, to.join(file_name))?;
        }
    } else {
        // the files and folders to be copied
        // this is better than using recursion in the case of stack overflows
        let mut copy_queue: Vec<(DirEntry, PathBuf)> = Vec::new();
        for dir_entry in read_dir(&from)? {
            if let Ok(dir_entry) = dir_entry {
                let file_name = dir_entry.file_name();
                if keep_num == 1 {
                    copy_queue.push((
                        dir_entry,
                        to.join(from.file_name().unwrap()).join(file_name),
                    ));
                } else {
                    let time_dir = to
                        .join(from.file_name().unwrap())
                        .join(&now)
                        .join(file_name);
                    copy_queue.push((dir_entry, time_dir));
                }
            }
        }
        // creates the target folder if it doesn't exist
        if keep_num == 1 && File::open(to.join(from.file_name().unwrap())).is_err() {
            create_dir(to.join(from.file_name().unwrap()))?;
        } else if keep_num > 1 {
            create_dir(to.join(from.file_name().unwrap()).join(&now))?;
            clear_old(&to.join(from.file_name().unwrap()), keep_num)
        }

        while !copy_queue.is_empty() {
            let entry = copy_queue.pop().unwrap();
            let info: Metadata = entry.0.metadata().unwrap();
            if info.is_file() {
                if ignored(&entry.0.path(), &info, &ignored_files, &ignored_folders) {
                    continue;
                }
                if check_timestamp {
                    let copied_file = File::open(&entry.1);
                    if let Ok(copied_file) = copied_file {
                        if entry.0.metadata().unwrap().modified().unwrap()
                            < copied_file.metadata().unwrap().modified().unwrap()
                        {
                            continue;
                        }
                    }
                }
                if copy(entry.0.path(), &entry.1).is_ok() {
                    num += 1;
                }
            } else {
                let entry_path = entry.0.path();
                let striped_path = entry_path.strip_prefix(&from);
                if let Ok(striped_path) = striped_path {
                    if ignored(
                        &striped_path.to_path_buf(),
                        &info,
                        &ignored_files,
                        &ignored_folders,
                    ) {
                        continue;
                    }
                }
                if File::open(&entry.1).is_err() {
                    create_dir(&entry.1).is_ok();
                }
                if let Ok(dir_entries) = read_dir(entry.0.path()) {
                    for e in dir_entries {
                        if let Ok(e) = e {
                            let target_path: PathBuf = entry.1.join(e.file_name());
                            copy_queue.push((e, target_path));
                        }
                    }
                }
            }
        }
    }

    Ok(num)
}

/// Clears the oldest backups based on keep num
fn clear_old(directory: &PathBuf, keep_num: i32) {
    let file_opt = File::open(directory);
    if let Ok(file) = file_opt {
        if file.metadata().unwrap().is_file() {
            return;
        }
        if let Ok(entries) = read_dir(directory) {
            let mut dir_names: Vec<std::ffi::OsString> = entries
                .map(|entry: Result<DirEntry>| {
                    if let Ok(entry) = entry {
                        entry.file_name()
                    } else {
                        panic!("Could not read the directory!")
                    }
                })
                .collect();
            if dir_names.len() < 2 {
                return;
            }

            while dir_names.len() > keep_num as usize {
                let mut new_min = dir_names[0].clone();
                let mut index = 0;
                for (i, item) in dir_names.iter().enumerate().skip(1) {
                    if *item < new_min {
                        new_min = item.clone();
                        index = i;
                    }
                }
                dir_names.remove(index);
                remove_dir_all(directory.join(new_min)).is_ok();
            }
        }
    }
}
