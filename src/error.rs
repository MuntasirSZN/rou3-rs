//! Defines the error types used throughout the `rou3` crate.

use thiserror::Error;

/// The primary error type for `rou3` operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RouterError {
    /// Represents an error that occurred due to an invalid path pattern.
    /// This could be due to malformed segments or unsupported syntax.
    #[error("invalid path pattern: {0}")]
    InvalidPath(String),

    /// Indicates that no route could be found matching the given method and path.
    #[error("route not found for method '{method}' and path '{path}'")]
    RouteNotFound {
        /// The HTTP method for which the route was not found.
        method: String,
        /// The path for which the route was not found.
        path: String,
    },

    /// Represents an error when attempting to parse or interpret a segment of a path.
    #[error("invalid segment '{segment}': {reason}")]
    InvalidSegment {
        /// The problematic segment.
        segment: String,
        /// The reason why the segment is invalid.
        reason: String,
    },
}
