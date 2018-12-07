use crate::copy_operation::CopyOperation;
use std::io::Error;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct LocalCopy;

impl CopyOperation for LocalCopy {
    type PathType = PathBuf;
    type Error = Error;

    fn copy(
        _from: Self::PathType,
        _to: Self::PathType,
        _check_timestamp: bool,
        _num_threads: i32,
    ) -> Result<u32, Self::Error> {
        unimplemented!();
    }

    fn sync(
        _from: Self::PathType,
        _to: Self::PathType,
        _num_threads: i32,
    ) -> Result<u32, Self::Error> {
        unimplemented!();
    }
}
