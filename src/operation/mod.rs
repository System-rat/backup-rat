// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Contains helper methods and structs for backup operations
//! such as checking timestamps, copying to targets and restoring

use std::fs::{copy, create_dir, read_dir};
use std::fs::{DirEntry, File, Metadata};
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

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

/// Copies a or folder file to a destination
/// Checking timestamps to override or not
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
pub fn copy_to(from: &PathBuf, to: &PathBuf, check_timestamp: bool) -> Result<i32> {
    let mut num: i32 = 0;
    let from_file: File = File::open(from)?;

    // TODO: Fix this mess
    if from_file.metadata().unwrap().is_file() {
        if let Some(file_name) = from.file_name() {
            if let Ok(to_file) = File::open(to.join(file_name)) {
                if check_timestamp {
                    if from_file.metadata().unwrap().modified().unwrap()
                        > to_file.metadata().unwrap().modified().unwrap()
                    {
                        num += 1;
                        copy(from, to.join(file_name))?;
                    }
                } else {
                    num += 1;
                    copy(from, to.join(file_name))?;
                }
            } else {
                num += 1;
                copy(from, to.join(file_name))?;
            }
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
                            > copied_file.metadata().unwrap().modified().unwrap()
                            && copy(entry.0.path(), &entry.1).is_ok()
                        {
                            num += 1;
                        }
                    } else if copy(entry.0.path(), &entry.1).is_ok() {
                        num += 1;
                    }
                } else if copy(entry.0.path(), &entry.1).is_ok() {
                    num += 1;
                }
            } else {
                if File::open(&entry.1).is_err() {
                    if let Ok(_) = create_dir(&entry.1) {
                        num += 1;
                    }
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

/// Backs up the target
///
/// # Parameters
/// - target: a reference to the target in question
/// # Returns
/// the number of files copied (and folders created)
/// # Error
/// Returns an error if:
/// - the target is unavailable (ex. unmounted drive)
/// - the backup target can't be read
/// - the destination can't be written to
///
/// # TODO
/// - Actually use the keep_num variable of the target
pub fn copy_to_target(target: &BackupTarget) -> Result<i32> {
    // checks
    if File::open(&target.target_path).is_err() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "The destination is unavailable!",
        ));
    }
    let res = copy_to(&target.path, &target.target_path, !&target.always_copy)?;
    Ok(res)
}

#[cfg(test)]
mod operation_tests {
    use std::fs::remove_dir_all;

    #[test]
    fn size_of() {
        let path = ::std::env::current_dir()
            .unwrap()
            .join("test_resources/sub_folder");
        assert_eq!(super::size_of(&path).unwrap(), 14);
    }

    #[test]
    fn copy() {
        let path = ::std::env::current_dir()
            .unwrap()
            .join("test_resources/sub_folder");
        let target = ::std::env::current_dir().unwrap().join("test_resources/bk");
        remove_dir_all(&target.join("sub_folder")).is_ok();
        super::copy_to(&path, &target, false).unwrap();
        assert_eq!(14, super::size_of(&path).unwrap());
    }

}
