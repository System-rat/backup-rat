use crate::config::BackupTarget;
use crate::copy_operation::CopyOperation;
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct LocalCopy {
    target: BackupTarget,
}

impl CopyOperation for LocalCopy {
    type Error = Error;

    fn prepare_target(&mut self, target: crate::config::BackupTarget) -> Result<(), Self::Error> {
        self.target = target;
        match File::open(&self.target.path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn file_list(&self) -> Vec<PathBuf> {
        unimplemented!()
    }

    fn copy_method(&self) -> Box<FnOnce(PathBuf, PathBuf) -> Result<(), Error>> {
        unimplemented!()
    }
}