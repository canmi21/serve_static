/* src/lib.rs */
#![doc = "Headless utilities for static file serving."]

/// Unified error types for serve_static.
pub mod error;
/// Directory entry data model and sorting utilities.
pub mod listing;
/// MIME type detection and ETag generation.
pub mod mime;
/// Safe path resolution with directory traversal protection.
pub mod path;
/// HTTP Range header parsing (RFC 9110).
pub mod range;

pub use error::Error;
