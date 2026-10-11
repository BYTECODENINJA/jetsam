//! Error types for `jetsam-core`.

use thiserror::Error;

/// Errors produced by core operations.
#[derive(Debug, Error)]
pub enum CoreError {
    /// The requested package is not present in the local database.
    #[error("package not found: {0}")]
    PackageNotFound(String),

    /// A string was not a valid package identifier.
    #[error("invalid package identifier: {0}")]
    InvalidPackageName(String),

    /// The dependency graph was malformed or a lookup failed.
    #[error("dependency graph error: {0}")]
    Graph(String),

    /// Filesystem access failed.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// A libalpm call failed. The original error is flattened to a string
    /// because `alpm::Error` is not guaranteed to be `Send + Sync` across
    /// versions and we never want to leak C pointers into our error type.
    #[cfg(feature = "libalpm")]
    #[error("libalpm error: {0}")]
    Alpm(String),
}

/// Convenience result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(feature = "libalpm")]
impl From<alpm::Error> for CoreError {
    fn from(err: alpm::Error) -> Self {
        CoreError::Alpm(err.to_string())
    }
}
