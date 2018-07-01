// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Contains helper methods and structs for backup operations
//! such as checking timestamps, copying to targets and restoring

use std::fs::{copy, create_dir, read_dir, remove_file};
use std::fs::{File, Metadata, OpenOptions};
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

/// Checks if the file or folder has read permission and write
/// permission if the option is set
///
/// # Parameters
/// - path: path to the file or folder in question
/// - check_write: wether to check write permissions or not
///
/// # Note
/// This function returns false if **EVEN** 1 file or folder
/// is read-only when *check_write* is on and returns false
/// if **EVEN** 1 file or folder is not readable
pub fn check_perm(path: &PathBuf, check_write: bool) -> bool {
    let file: Result<File> = File::open(path);
    let mut perms: OpenOptions = OpenOptions::new();
    perms.read(true).write(true).create(true);
    if let Ok(file) = file {
        let metadata: &Metadata = &file.metadata().unwrap();
        if metadata.is_dir() {
            if check_write && !can_create(path) {
                return false;
            }
            let files = read_dir(path);
            if let Ok(files) = files {
                for file in files {
                    if let Ok(file) = file {
                        if !check_perm(&file.path(), check_write) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                return true;
            } else {
                return false;
            }
        } else {
            if check_write && perms.open(path).is_err() {
                return false;
            }
            return true;
        }
    }
    false
}

/// Checks to see if a directory has create permission
///
/// # Note
/// Why is there no way to do this in the standard library c'mon
/// Rust you're better than this (if there is submit a pull request)
pub fn can_create(path: &PathBuf) -> bool {
    // HACK: Only valid way to check if a directory is writable in Rust
    // damn it
    if let Ok(_f) = File::create(path.join("brdt.backup_rat")) {
        remove_file(path.join("brdt.backup_rat")).unwrap();
        return true;
    } else {
        return false;
    }
}

/// Copies a or folder file to a destination
/// Checking timestamps to override or not
///
/// # Parameters
/// - from: the file or folder to copy
/// - to: the exact path of the file or the parent dir of the folder
/// (I don't know why I did it this way)
/// - check_timestamp: wether to check file modification before copy
/// or always copy
///
/// # Error
/// Returns an error if there were fs error during copying
pub fn copy_to(from: &PathBuf, to: &PathBuf, check_timestamp: bool) -> Result<()> {
    if !check_perm(from, false) {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            format!("Could not get full read access: {}", from.display()),
        ));
    }
    if !can_create(to) {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            format!(
                "Could not obtain write permission for destination: from: {} to: {}",
                from.display(),
                to.display()
            ),
        ));
    }
    copy_to_inner(from, to, check_timestamp)
}

/// This is the actual copy function meant for recursion, the previous one
/// Is there to check if the initial dir is readable and the target writable
fn copy_to_inner(from: &PathBuf, to: &PathBuf, check_timestamp: bool) -> Result<()> {
    let file: &File = &File::open(from)?;
    let meta: &Metadata = &file.metadata().unwrap();
    if meta.is_file() {
        if check_timestamp {
            let target_file: Result<File> = File::open(to);
            if let Ok(target_file) = target_file {
                let target_meta: &Metadata = &target_file.metadata().unwrap();
                if target_meta.modified().unwrap() < meta.modified().unwrap() {
                    copy(from, to)?;
                } else {
                    return Ok(());
                }
            } else {
                copy(from, to)?;
            }
        } else {
            copy(from, to)?;
        }
    } else {
        let target_file: Result<File> = File::open(to.join(from.file_name().unwrap()));
        if !target_file.is_ok() {
            create_dir(to.join(PathBuf::from(from.file_name().unwrap())))?;
        }
        for f in read_dir(from)? {
            let f = f?;
            if f.metadata().unwrap().is_dir() {
                copy_to_inner(
                    &f.path(),
                    &to.join(PathBuf::from(from.file_name().unwrap())),
                    check_timestamp,
                )?;
            } else {
                copy_to_inner(
                    &f.path(),
                    &to.join(PathBuf::from(from.file_name().unwrap()))
                        .join(f.file_name()),
                    check_timestamp,
                )?;
            }
        }
    }
    Ok(())
}

/// Backs up the target
///
/// # Parameters
/// - target: a reference to the target in question
///
/// # Error
/// Returns an error if:
/// - the target is unavailable (ex. unmounted drive)
/// - the backup target can't be read
/// - the destination can't be written to
///
/// # TODO
/// - Actually use the keep_num variable of the target
pub fn copy_to_target(target: &BackupTarget) -> Result<()> {
    // Checks
    if !check_perm(&target.path, false) {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "Can't read backup target.",
        ));
    }
    if !File::open(&target.target_path).is_ok() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Backup location unavailable.",
        ));
    }
    if !can_create(&target.target_path) {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "Can't write to backup location.",
        ));
    }

    // Copy actions
    // If its a folder we just copy it to the destination
    // If its a file we copy it to the destination with its
    // original file name (blame the copy_to function)
    let from: File = File::open(&target.path)?;
    if from.metadata().unwrap().is_file() {
        copy_to(
            &target.path,
            &target.target_path.join(&target.path.file_name().unwrap()),
            !&target.always_copy,
        )?;
    } else {
        copy_to(&target.path, &target.target_path, !&target.always_copy)?;
    }

    Ok(())
}

#[cfg(test)]
mod operation_tests {
    use super::check_perm;
    use std::env::current_dir;
    use std::fs::remove_dir_all;
    use std::path::PathBuf;

    #[test]
    fn size_of() {
        let path = ::std::env::current_dir()
            .unwrap()
            .join("test_resources/sub_folder");
        assert_eq!(super::size_of(&path).unwrap(), 14);
    }

    #[test]
    fn permissions() {
        assert!(!check_perm(&PathBuf::from("/"), true));
        assert!(!check_perm(&PathBuf::from("/"), false));
        assert!(check_perm(
            &current_dir().unwrap().join("test_resources"),
            true
        ));
    }

    #[test]
    fn copy() {
        let path = ::std::env::current_dir()
            .unwrap()
            .join("test_resources/sub_folder");
        let target = ::std::env::current_dir().unwrap().join("test_resources/bk");
        remove_dir_all(&target.join(PathBuf::from("sub_folder"))).is_ok();
        super::copy_to(&path, &target, false).unwrap();
        assert_eq!(14, super::size_of(&path).unwrap());
    }

}
