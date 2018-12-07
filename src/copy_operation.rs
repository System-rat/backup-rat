/// A trait for copy operations done on targets
/// TODO: Implement current copy operations using this trait
pub trait CopyOperation {
    /// The type of the path variables
    type PathType;
    /// The type of error returned by the operations
    type Error;

    /// Copies an item from one place to another.
    ///
    /// #Parameters
    ///
    /// -from: The path of the item
    /// -to: The path of the destination
    /// -check_timestamp: Whether to check if the file was changed
    /// -num_threads: The number of threads to use (none if 1 or 0)
    ///
    /// #Returns
    /// The umber of items
    ///
    /// #Errors
    /// Returns an error if the operation was not successful
    fn copy(
        from: Self::PathType,
        to: Self::PathType,
        check_timestamp: bool,
        num_threads: i32,
    ) -> Result<u32, Self::Error>;

    /// Synchronizes the item and its destination instead of copying.
    /// Eg. if one item exists in the destination or origin it is copied
    /// to the other.
    ///
    /// #Parameters
    ///
    /// -from: The path of the item
    /// -to: The path of the destination
    /// -num_threads: The number of threads to use (none if 1 or 0)
    ///
    /// #Returns
    /// The umber of items
    ///
    /// #Errors
    /// Returns an error if the operation was not successful
    fn sync(from: Self::PathType, to: Self::PathType, num_threads: i32)
        -> Result<u32, Self::Error>;
}
