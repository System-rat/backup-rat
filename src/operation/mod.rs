// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Contains helper methods and structs for backup operations
//! such as checking timestamps, copying to targets and restoring

use std::fs::{copy, create_dir, create_dir_all, read_dir};
use std::fs::{DirEntry, File, Metadata};
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

use regex::Regex;

use super::config::BackupTarget;

/// Returns the size of the directory or file in bytes
///
/// # Parameters
/// - path: a reference of a path to the file or folder in question
///
/// # Error
/// returns and error if the file or folder can't be read
pub fn size_of(path: &PathBuf) -> Result<u64> {
    let file: &File = &File::open(path)?;
    let metadata: Metadata = file.metadata().unwrap();
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut sum: u64 = 0;
    for file in read_dir(path)? {
        if let Ok(file) = file {
            let meta: &Metadata = &file.metadata().unwrap();
            let spath: &PathBuf = &file.path();
            if meta.is_file() {
                sum += meta.len();
            } else {
                if let Ok(size) = size_of(spath) {
                    sum += size;
                }
            }
        } else {
            continue;
        }
    }
    Ok(sum)
}

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
    ignored_files: &Vec<String>,
    ignored_folders: &Vec<String>,
) -> bool {
    // TODO: implement
    if metadata.is_file() {
        for filter in ignored_files {
            if filter.starts_with("r#") {
                let r = Regex::new(&filter[2..]);
                if let Ok(r) = r {
                    if r.is_match(path.file_name().unwrap().to_str().unwrap()) {
                        return true;
                    }
                }
            } else {
                if path.file_name().unwrap().to_str().unwrap() == filter {
                    return true;
                }
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
            } else {
                if path.as_os_str().to_str().unwrap() == filter {
                    return true;
                }
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
///
/// TODO: implement `ignored` for directories
pub fn copy_to(target: &BackupTarget) -> Result<i32> {
    let from = &target.path;
    let to = &target.target_path;
    let check_timestamp = !target.always_copy;
    let ignored_files = &target.ignore_files;
    let ignored_folders = &target.ignore_folders;
    let mut num: i32 = 0;
    let from_file: File = File::open(from)?;

    if from_file.metadata().unwrap().is_file() {
        if let Some(file_name) = from.file_name() {
            if let Ok(to_file) = File::open(to.join(file_name)) {
                if check_timestamp
                    && from_file.metadata().unwrap().modified().unwrap()
                        < to_file.metadata().unwrap().modified().unwrap()
                {
                    return Ok(0);
                }
            }
            if ignored(
                from,
                &from_file.metadata().unwrap(),
                ignored_files,
                ignored_folders,
            ) {
                return Ok(0);
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
                copy_queue.push((
                    dir_entry,
                    to.join(from.file_name().unwrap()).join(file_name),
                ));
            }
        }
        // creates the target folder if it doesn't exist
        if File::open(to.join(from.file_name().unwrap())).is_err() {
            create_dir(to.join(from.file_name().unwrap()))?;
        }

        while !copy_queue.is_empty() {
            let entry = copy_queue.pop().unwrap();
            let info: Metadata = entry.0.metadata().unwrap();
            if info.is_file() {
                if check_timestamp {
                    let copied_file = File::open(&entry.1);
                    if let Ok(copied_file) = copied_file {
                        if entry.0.metadata().unwrap().modified().unwrap()
                            < copied_file.metadata().unwrap().modified().unwrap()
                        {
                            continue;
                        }
                        if ignored(&entry.0.path(), &info, &ignored_files, &ignored_folders) {
                            continue;
                        }
                    }
                }
                if copy(entry.0.path(), &entry.1).is_ok() {
                    num += 1;
                }
            } else {
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
///
/// TODO: implement `ignored` for directories
pub fn threaded_copy_to(target: &BackupTarget, num_threads: i32) -> Result<i32> {
    let from = &target.path;
    let to = &target.target_path;
    let check_timestamp = !target.always_copy;
    let ignored_files = &target.ignore_files;
    let ignored_folders = &target.ignore_folders;
    if let Ok(file) = File::open(from) {
        if file.metadata().unwrap().is_file() {
            copy(from, to.join(from.file_name().unwrap()))?;
            return Ok(1);
        }
    }

    if File::open(to).is_err() {
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
                    if create_dir_all(to.parent().unwrap()).is_ok() {
                        if copy(from, to).is_ok() {
                            num += 1;
                        }
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
            read_files.push((dir_entry.path(), to.clone().join(from.file_name().unwrap())));
        }
    }

    // WARNING: This code is confusing...
    while !read_files.is_empty() {
        let (file_path, file_parent) = read_files.pop().unwrap();
        let file = File::open(&file_path);
        if let Ok(file) = file {
            let metadata = file.metadata().unwrap();
            if metadata.is_dir() {
                if let Ok(entries) = read_dir(&file_path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if let Ok(f) = File::open(entry.path()) {
                                if f.metadata().unwrap().is_dir() {
                                    read_files.push((
                                        entry.path(),
                                        file_parent.join(file_path.file_name().unwrap()),
                                    ));
                                } else {
                                    read_files.push((
                                        entry.path(),
                                        file_parent.join(file_path.file_name().unwrap()),
                                    ));
                                }
                            }
                        }
                    }
                }
            } else {
                if check_timestamp {
                    let target_file_path = file_parent.clone().join(file_path.file_name().unwrap());
                    if let Ok(target_file) = File::open(target_file_path) {
                        if target_file.metadata().unwrap().modified().unwrap()
                            > metadata.modified().unwrap()
                        {
                            continue;
                        }
                        if ignored(
                            &file_path,
                            &target_file.metadata().unwrap(),
                            &ignored_files,
                            &ignored_folders,
                        ) {
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

    let res;
    if threads > 1 {
        res = threaded_copy_to(target, threads)?;
    } else {
        res = copy_to(target)?;
    }
    Ok(res)
}
