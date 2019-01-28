use crate::config::BackupTarget;
use crate::copy_operation::CopyOperation;
use std::fs::read_dir;
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct LocalCopy {
    target: BackupTarget,
}

impl CopyOperation for LocalCopy {
    type Error = Error;

    fn copy_method(&self) -> Box<FnMut(PathBuf, PathBuf) -> Result<(), Error>> {
        Box::new(|from: PathBuf, to: PathBuf| {
            println!("{}: {}", from.display(), to.display());
            Ok(())
        })
    }

    fn file_list(&self) -> Vec<(PathBuf, PathBuf)> {
        let mut files: Vec<(PathBuf, PathBuf)> = Vec::new();
        if let Ok(dir) = File::open(&self.target.path) {
            if dir.metadata().unwrap().is_file() {
                files.push((
                    self.target.path.clone(),
                    self.target
                        .target_path
                        .join(self.target.path.file_name().unwrap()),
                ));
            } else {
                let mut dirs_to_read: Vec<PathBuf> = Vec::new();
                dirs_to_read.push(self.target.path.clone());
                while !dirs_to_read.is_empty() {
                    if let Ok(dirs) = read_dir(dirs_to_read.pop().unwrap()) {
                        dirs.for_each(|dir_entry| {
                            if let Ok(dir_entry) = dir_entry {
                                if dir_entry.metadata().unwrap().is_dir() {
                                    dirs_to_read.push(dir_entry.path());
                                } else {
                                    files.push((
                                        dir_entry.path(),
                                        self.target.target_path.join(
                                            dir_entry
                                                .path()
                                                .strip_prefix(self.target.path.parent().unwrap())
                                                .unwrap(),
                                        ),
                                    ));
                                }
                            }
                        });
                    }
                }
            }
        }
        files
    }
}

impl LocalCopy {
    pub fn prepare_target(target: crate::config::BackupTarget) -> Result<Self, Error> {
        let operation = LocalCopy { target };
        match File::open(&operation.target.path) {
            Ok(_) => Ok(operation),
            Err(e) => Err(e),
        }
    }
}
