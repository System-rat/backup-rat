/// A trait for copy operations done on targets
/// TODO: Implement current copy operations using this trait
pub trait CopyOperation {
    /// The type of error returned by the operations
    type Error: std::error::Error;

    /// Performs the copy operation for the target.
    ///
    /// # Parameters
    /// - threads: the number of threads (<2 is single threaded)
    ///
    /// # Returns
    /// The umber of items
    ///
    /// # Errors
    /// Returns an error if the operation was not successful
    fn perform_copy(threads: u32) -> Result<u32, Self::Error>;

    /// Prepares the target
    ///
    /// # Parameters
    /// - target: The config target to be prepared
    ///
    /// # Errors
    /// Returns an error if the preparation failed
    fn prepare_target(target: crate::config::BackupTarget) -> Result<(), Self::Error>;
}
