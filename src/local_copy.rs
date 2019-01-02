use crate::copy_operation::CopyOperation;
use std::io::Error;

#[allow(dead_code)]
pub struct LocalCopy;

impl CopyOperation for LocalCopy {
    type Error = Error;

    fn prepare_target(_target: crate::config::BackupTarget) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn perform_copy(_threads: u32) -> Result<u32, Self::Error> {
        unimplemented!()
    }
}
