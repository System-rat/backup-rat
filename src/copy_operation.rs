/// A trait for copy operations done on targets
/// TODO: Implement current copy operations using this trait
///
/// # Note
/// These methods exist to make copying threaded
pub trait CopyOperation {
    /// The type of error returned by the operations
    type Error: std::error::Error;

    /// Gets the method that does the file copying
    ///
    /// # FnOne parameters
    /// 1. Where to get the file
    /// 2. Where to put it
    ///
    /// # Errors
    /// Returns an error if the operation fails
    fn copy_method(
        &self,
    ) -> Box<FnOnce(std::path::PathBuf, std::path::PathBuf) -> Result<(), Self::Error>>;

    /// Gets a list of files to be copied
    ///
    /// # Returns
    /// The files to be copied
    fn file_list(&self) -> Vec<std::path::PathBuf>;

    /// Prepares the target
    ///
    /// # Parameters
    /// - target: The config target to be prepared
    ///
    /// # Errors
    /// Returns an error if the preparation failed
    fn prepare_target(&mut self, target: crate::config::BackupTarget) -> Result<(), Self::Error>;
}
