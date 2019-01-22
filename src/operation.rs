// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Contains helper methods and structs for backup operations
//! such as checking timestamps, copying to targets and restoring

use std::fs::{copy, create_dir, create_dir_all, read_dir, remove_dir_all};
use std::fs::{DirEntry, File, Metadata};
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

use regex::Regex;

use crate::config::BackupTarget;

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
/// - from: the file or folder to copy
/// - to: the parent dir of the *from* object
/// - check_timestamp: wether to check file modification before copy
/// or always copy
///
/// # Returns
/// Returns an the number of copied files
///
/// # Error
/// Returns an error if the target could not be written to or the *from* target could
/// not be read
pub fn copy_to(target: &BackupTarget) -> Result<i32> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let from = &target.path;
    let to = &target.target_path;
    let check_timestamp = if target.keep_num == 1 {
        !target.always_copy
    } else {
        false
    };
    let ignored_files = &target.ignore_files;
    let ignored_folders = &target.ignore_folders;
    let mut num: i32 = 0;
    let from_file: File = File::open(from)?;

    if from_file.metadata().unwrap().is_file() {
        if let Some(file_name) = from.file_name() {
            if ignored(
                from,
                &from_file.metadata().unwrap(),
                ignored_files,
                ignored_folders,
            ) {
                return Ok(0);
            }
            if let Ok(to_file) = File::open(to.join(file_name)) {
                if check_timestamp
                    && from_file.metadata().unwrap().modified().unwrap()
                        < to_file.metadata().unwrap().modified().unwrap()
                {
                    return Ok(0);
                }
            }
            num += 1;
            copy(from, to.join(file_name))?;
        }
    } else {
        // the files and folders to be copied
        // this is better than using recursion in the case of stack overflows
        let mut copy_queue: Vec<(DirEntry, PathBuf)> = Vec::new();
        for dir_entry in read_dir(from)? {
            if let Ok(dir_entry) = dir_entry {
                let file_name = dir_entry.file_name();
                if target.keep_num == 1 {
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
        if target.keep_num == 1 && File::open(to.join(from.file_name().unwrap())).is_err() {
            create_dir(to.join(from.file_name().unwrap()))?;
        } else if target.keep_num > 1 {
            create_dir(to.join(from.file_name().unwrap()).join(&now))?;
            clear_old(&to.join(from.file_name().unwrap()), target.keep_num)
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
                let striped_path = entry_path.strip_prefix(from);
                if let Ok(striped_path) = striped_path {
                    if ignored(
                        &striped_path.to_path_buf(),
                        &info,
                        ignored_files,
                        ignored_folders,
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

enum Command {
    Terminate,
    Copy(PathBuf, PathBuf),
}

/// Copies a folder or file to a destination using multiple threads
/// whilst also checking timestamps to override or not
///
/// # Parameters
/// - from: the file or folder to copy
/// - to: the parent dir of the *from* object
/// - check_timestamp: wether to check file modification before copy
/// or always copy
///
/// # Returns
/// Returns an the number of copied files
///
/// # Error
/// Returns an error if the target could not be written to or the *from* target could
/// not be read
///
/// # Notes
/// - If the target is a file it will use no threads
pub fn threaded_copy_to(target: &BackupTarget, num_threads: i32) -> Result<i32> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let from = &target.path;
    let to = &target.target_path;
    let check_timestamp = if target.keep_num == 1 {
        !target.always_copy
    } else {
        false
    };
    let ignored_files = &target.ignore_files;
    let ignored_folders = &target.ignore_folders;
    if let Ok(file) = File::open(from) {
        if file.metadata().unwrap().is_file() {
            copy(from, to.join(from.file_name().unwrap()))?;
            return Ok(1);
        }
    }

    if File::open(&to).is_err() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "The destination is unavailable!",
        ));
    }
    let (sender, receiver) = channel::<Command>();
    let arc_receiver = Arc::new(Mutex::new(receiver));
    let mut threads: Vec<JoinHandle<i32>> = Vec::new();
    let mut num: i32 = 0;
    for _ in 1..num_threads {
        let receiver = Arc::clone(&arc_receiver);
        threads.push(spawn(move || {
            let mut num = 0;
            loop {
                let command = receiver.lock().unwrap().recv().unwrap();
                if let Command::Terminate = command {
                    break;
                } else if let Command::Copy(from, to) = command {
                    if create_dir_all(to.parent().unwrap()).is_ok() && copy(from, to).is_ok() {
                        num += 1;
                    }
                }
            }
            num
        }));
    }
    let mut read_files: Vec<(PathBuf, PathBuf)> = Vec::new();

    if File::open(&to.join(from.file_name().unwrap())).is_err() {
        create_dir(&to.join(from.file_name().unwrap()))?;
    }

    for dir_entry in read_dir(from)? {
        if let Ok(dir_entry) = dir_entry {
            if target.keep_num == 1 {
                read_files.push((dir_entry.path(), to.clone().join(from.file_name().unwrap())));
            } else {
                read_files.push((
                    dir_entry.path(),
                    to.clone().join(from.file_name().unwrap()).join(&now),
                ));
            }
        }
    }

    if target.keep_num > 1 {
        create_dir_all(to.clone().join(from.file_name().unwrap()).join(&now)).is_ok();
        clear_old(&to.join(from.file_name().unwrap()), target.keep_num);
    }

    // WARNING: This code is confusing...
    while !read_files.is_empty() {
        let (file_path, file_parent) = read_files.pop().unwrap();
        let file = File::open(&file_path);
        if let Ok(file) = file {
            let metadata = file.metadata().unwrap();
            if metadata.is_dir() {
                let striped_path = file_path.strip_prefix(from);
                if let Ok(striped_path) = striped_path {
                    if ignored(
                        &striped_path.to_path_buf(),
                        &metadata,
                        ignored_files,
                        ignored_folders,
                    ) {
                        continue;
                    }
                }
                if let Ok(entries) = read_dir(&file_path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if File::open(entry.path()).is_ok() {
                                read_files.push((
                                    entry.path(),
                                    file_parent.join(file_path.file_name().unwrap()),
                                ));
                            }
                        }
                    }
                }
            } else {
                if check_timestamp {
                    let target_file_path = file_parent.clone().join(file_path.file_name().unwrap());
                    if ignored(&file_path, &metadata, &ignored_files, &ignored_folders) {
                        continue;
                    }
                    if let Ok(target_file) = File::open(target_file_path) {
                        if target_file.metadata().unwrap().modified().unwrap()
                            > metadata.modified().unwrap()
                        {
                            continue;
                        }
                    }
                }
                sender
                    .send(Command::Copy(
                        file_path.clone(),
                        file_parent.join(file_path.file_name().unwrap()),
                    ))
                    .is_ok();
            }
        }
    }

    for _ in 1..num_threads {
        sender.send(Command::Terminate).is_ok();
    }

    for handle in threads {
        num += handle.join().unwrap();
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

/// Backs up the target
///
/// # Parameters
/// - target: a reference to the target in question
///
/// # Returns
/// the number of files copied (and folders created)
///
/// # Error
/// Returns an error if:
/// - the target is unavailable (ex. unmounted drive)
/// - the backup target can't be read
/// - the destination can't be written to
///
/// # TODO
/// - Actually use the keep_num variable of the target
pub fn copy_to_target(target: &BackupTarget, threads: i32) -> Result<i32> {
    // checks
    if File::open(&target.target_path).is_err() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "The destination is unavailable!",
        ));
    }

    if threads > 1 {
        Ok(threaded_copy_to(target, threads)?)
    } else {
        Ok(copy_to(target)?)
    }
}
